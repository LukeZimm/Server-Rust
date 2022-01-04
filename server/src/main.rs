use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

const HEADER_SIZE: usize = 8;

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; HEADER_SIZE];
    while match stream.read(&mut data) {
        Ok(size) => {
            // stream.write(&data[0..size]).unwrap();
            let len = from_utf8(&data).unwrap().parse::<usize>().unwrap();
            let mut data = vec![0 as u8; len]; // read header
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    let msg = from_utf8(&data).unwrap();
                    println!("Recieved: {}", msg);
                    let msg = msg.as_bytes();
                    send(&mut stream, msg);
                }
                Err(e) => {
                    println!("Failed to recieve data: {}", e);
                }
            }
            true
        }
        Err(_) => {
            println!(
                "An error has occured, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    const PORT: &str = "3333";
    let listener = TcpListener::bind(String::from("127.0.0.1:") + PORT).unwrap();
    // accept connection and process them, spawning a new thread for each one
    println!("Server listening on PORT {}", PORT);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                // connection failed
            }
        }
    }
    // close the socket server
    drop(listener);
}

fn send(stream: &mut TcpStream, msg: &[u8]) {
    stream
        .write(pad_front(&msg.len().to_string(), HEADER_SIZE, "0").as_bytes())
        .unwrap();
    stream.write(msg).unwrap();
}

fn pad_front(msg: &str, len: usize, ch: &str) -> String {
    let mut out = String::new();
    for i in msg.len()..len {
        out += ch;
    }
    out + msg
}
fn pad_back(msg: &str, len: usize, ch: &str) -> String {
    let mut out = String::from(msg);
    for i in msg.len()..len {
        out += ch;
    }
    out
}
