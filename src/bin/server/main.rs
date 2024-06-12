use chat::{
    msg::{FromClient, FromServer},
    room_id::RoomId,
    room_name::RoomName,
};
use lazy_static::lazy_static;
use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc, vec};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::Mutex,
};

type Rooms = Arc<(Mutex<Vec<SocketAddr>>, RoomName)>;

lazy_static! {
    static ref TO_SEND: Mutex<HashMap<SocketAddr, Mutex<Vec<FromServer>>>> =
        Mutex::new(HashMap::new());
    static ref ROOM_ID_TO_ROOM: Mutex<HashMap<RoomId, Rooms>> = Mutex::new(HashMap::new());
    static ref CLIENT_TO_ROOM: Mutex<HashMap<SocketAddr, Rooms>> = Mutex::new(HashMap::new());
    static ref NAMES: Mutex<HashMap<SocketAddr, Box<str>>> = Mutex::new(HashMap::new());
}

async fn send_all_from(write: &mut OwnedWriteHalf, client: SocketAddr) -> Result<(), Disconnected> {
    let lock = TO_SEND.lock().await;
    let Some(to_send) = lock.get(&client) else {
        return Ok(());
    };

    let mut to_send = to_send.lock().await;

    for msg in to_send.iter() {
        let mut msg = serde_json::to_string(msg).unwrap();
        msg.push('\n');

        // write.write_all(msg.as_bytes()).await.unwrap();

        match write.write_all(msg.as_bytes()).await {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::ConnectionReset => return Err(Disconnected),
            Err(err) => eprintln!("Error: {}", err),
        }

        println!("To {}: {}", client, msg.trim());
    }

    to_send.clear();

    Ok(())
}

async fn process_msg(msg: FromClient, addr: SocketAddr) -> FromServer {
    use FromClient::*;

    match msg {
        Name(name) => {
            NAMES.lock().await.insert(addr, name);
            FromServer::Renamed
        }
        Create(name) => {
            let clients = Arc::new((Mutex::new(vec![addr]), name.clone()));
            let room_id = RoomId::new(rand::random());

            let mut room_id_to_room = ROOM_ID_TO_ROOM.lock().await;
            room_id_to_room.insert(room_id, clients.clone());

            let mut client_to_room = CLIENT_TO_ROOM.lock().await;

            if let Some(old_room) = client_to_room.insert(addr, clients) {
                let mut lock = old_room.0.lock().await;
                if let Some(index) = lock.iter().position(|x| *x == addr) {
                    lock.remove(index);
                }
            }

            println!("Room `{}` created by {}", name, addr);

            FromServer::NewRoom(room_id)
        }
        Join(id) => {
            let lock = ROOM_ID_TO_ROOM.lock().await;
            let Some(room) = lock.get(&id) else {
                return FromServer::RoomNotFound;
            };

            {
                let mut lock = room.0.lock().await;
                lock.push(addr);
            }

            let mut lock = CLIENT_TO_ROOM.lock().await;
            if let Some(old_room) = lock.insert(addr, room.clone()) {
                let mut lock = old_room.0.lock().await;
                if let Some(index) = lock.iter().position(|x| *x == addr) {
                    lock.remove(index);
                }
            }

            println!("Client {} has joined `{}`", addr, room.1);

            FromServer::Joined(room.1.clone())
        }
        Leave => {
            let mut lock = CLIENT_TO_ROOM.lock().await;

            if let Some(room) = lock.remove(&addr) {
                let mut vec = room.0.lock().await;

                if let Some(index) = vec.iter().position(|x| *x == addr) {
                    vec.remove(index);
                }

                return FromServer::Left;
            }

            FromServer::NotJoined
        }
        Message(msg) => {
            let lock = CLIENT_TO_ROOM.lock().await;
            if let Some(room) = lock.get(&addr) {
                let clients = room.0.lock().await;
                let mut to_send = TO_SEND.lock().await;

                for client in clients.iter() {
                    let names = NAMES.lock().await;
                    let name = match names.get(&addr) {
                        Some(name) => name.as_ref(),
                        None => "unknown",
                    };

                    if let Some(v) = to_send.get(client) {
                        let mut msg_to_send = v.lock().await;
                        msg_to_send.push(FromServer::Message {
                            name: name.into(),
                            msg: msg.clone(),
                        });
                        continue;
                    }

                    to_send.insert(
                        *client,
                        Mutex::new(vec![FromServer::Message {
                            name: name.into(),
                            msg: msg.clone(),
                        }]),
                    );
                }

                return FromServer::Sent;
            }

            FromServer::NotJoined
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Disconnected;

async fn process_incoming(read: &mut OwnedReadHalf, addr: SocketAddr) {
    let mut read = BufReader::new(read);

    loop {
        let mut s = String::new();
        match read.read_line(&mut s).await {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::ConnectionReset => return,
            Err(err) => eprintln!("Error: {}", err),
        }

        let msg = serde_json::from_str(&s).unwrap();

        println!("From {}: {}", addr, s.trim());

        let response = process_msg(msg, addr).await;

        let mut to_send = TO_SEND.lock().await;
        match to_send.get_mut(&addr) {
            Some(vec) => vec.lock().await.push(response),
            None => _ = to_send.insert(addr, Mutex::new(vec![response])),
        }
    }
}

async fn process_conn(stream: TcpStream, addr: SocketAddr) {
    {
        let mut lock = NAMES.lock().await;
        lock.insert(addr, format!("user-{}", rand::random::<u32>()).into());
    }

    let (mut read, write) = stream.into_split();

    let handle = tokio::spawn(async move {
        let mut write = write;

        loop {
            if send_all_from(&mut write, addr).await == Err(Disconnected) {
                break;
            };
        }
    });

    process_incoming(&mut read, addr).await;

    handle.abort();

    println!("Client {} has disconnected", addr);
}

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind(chat::SERVER_ADDR).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Couldn't bind tcp socket: {}", e);
            return;
        }
    };

    println!("Listening on {}", chat::SERVER_ADDR);

    loop {
        let conn = match listener.accept().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Couldn't get client: {}", e);
                continue;
            }
        };

        println!("Connected with {}", conn.1);

        tokio::spawn(process_conn(conn.0, conn.1));
    }
}
