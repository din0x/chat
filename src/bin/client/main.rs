use chat::msg::{FromClient, FromServer};
use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::TcpStream,
    str::FromStr,
    thread,
};

mod io_sync;

fn process_incoming(msg: &FromServer) {
    match msg {
        FromServer::Joined(name) => io_sync::println(&format!("Joined: {}", name)),
        FromServer::Left => io_sync::println("Left room"),
        FromServer::Message { name, msg } => io_sync::println(&format!("{}: {}", name, msg)),
        FromServer::NewRoom(id) => io_sync::println(&format!("Created new room: {}", id)),
        FromServer::RoomNotFound => io_sync::eprintln("Couldn't find this room"),
        FromServer::NotJoined => io_sync::eprintln("You need to join to a room"),
        FromServer::Renamed => io_sync::println("Successfully changed name"),
        FromServer::Sent => {}
        FromServer::Error => io_sync::eprintln("Server couldn't respond"),
    }
}

fn main() {
    let mut stream = TcpStream::connect(chat::SERVER_ADDR).unwrap();
    let mut stream_clone = stream.try_clone().unwrap();

    // let response_buf = Arc::new(Mutex::new(Vec::new()));

    thread::spawn(move || loop {
        // let mut stream_clone = stream_clone;
        // let mut guard = stream_clone.lock().unwrap();
        // guard.set_nonblocking(false).unwrap();

        let mut buf = String::new();
        let mut reader = BufReader::new(stream_clone.by_ref());

        match reader.read_line(&mut buf) {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => {
                io_sync::eprintln(&format!("Error: {}", err));
            }
        }

        let Ok(msg): Result<FromServer, _> = serde_json::from_str(&buf) else {
            io_sync::eprintln(&format!("Couldn't deserialize response: `{}`", &buf));
            continue;
        };

        process_incoming(&msg);
        // response_buf.lock().unwrap().push(response);
    });

    loop {
        let buf = io_sync::input("> ");

        let message = match FromClient::from_str(buf.trim()) {
            Ok(message) => message,
            Err(err) => {
                io_sync::eprintln(&format!("Error: {}", err));
                continue;
            }
        };

        let Ok(mut serialized) = serde_json::to_string(&message) else {
            io_sync::eprintln("Couldn't serialize");
            continue;
        };

        serialized.push('\n');

        match stream.write_all(serialized.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                io_sync::eprintln(&format!("Couldn't write message: {:?}", err));
                continue;
            }
        }

        // io_sync::println(&format!("Sending message: {}", &serialized.trim_end()));
    }
}
