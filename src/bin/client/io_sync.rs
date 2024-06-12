use crossterm::{
    cursor::{MoveLeft, MoveToColumn},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    queue,
    terminal::{Clear, ClearType},
};
use std::{
    io::{stdout, Write},
    sync::Mutex,
};

// static WAS_READING: Mutex<bool> = Mutex::new(false);
static INPUT: Mutex<Option<String>> = Mutex::new(None);

pub fn input(prompt: &str) -> String {
    print!("{}", prompt);

    let mut buf = String::new();

    {
        let mut lock = INPUT.lock().unwrap();
        *lock = Some(prompt.into());
    }

    loop {
        match event::read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => {
                buf.push(ch);
                print!("{}", ch);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => {
                println!();
                let mut lock = INPUT.lock().unwrap();
                *lock = None;

                break buf;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) if !buf.is_empty() => {
                _ = queue!(stdout(), MoveLeft(1), Clear(ClearType::UntilNewLine));
                buf.pop();
            }
            _ => {}
        }

        stdout().flush().unwrap();

        let mut lock = INPUT.lock().unwrap();
        *lock = Some(format!("{}{}", prompt, buf));
    }
}

pub fn println(s: &str) {
    println_to(stdout(), s)
}

pub fn eprintln(s: &str) {
    println_to(stdout(), s)
}

fn println_to(mut target: impl Write, s: &str) {
    if let Some(prompt) = INPUT.lock().unwrap().as_ref() {
        queue!(target, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
        writeln!(target, "{}", s).unwrap();
        queue!(target, Clear(ClearType::CurrentLine)).unwrap();
        print!("{}", prompt);
    } else {
        writeln!(target, "{}", s).unwrap();
    }

    target.flush().unwrap();
}
