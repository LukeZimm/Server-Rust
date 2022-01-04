use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

const HEADER_SIZE: usize = 8;

fn main() {
    const PORT: &str = "3333";
    match TcpStream::connect(String::from("127.0.0.1:") + PORT) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port: {}", PORT);
            loop {
                let mut msg = String::new();
                println!("Message: ");
                let _ = std::io::stdin().read_line(&mut msg).unwrap();
                let msg = msg.as_bytes();
                send(&mut stream, msg);
                println!("Sent message, awaiting reply...");
                let mut data = [0 as u8; HEADER_SIZE]; // read header
                match stream.read_exact(&mut data) {
                    Ok(_) => {
                        let len = from_utf8(&data).unwrap().parse::<usize>().unwrap();
                        let mut data = vec![0 as u8; len]; // read header
                        match stream.read_exact(&mut data) {
                            Ok(_) => {
                                println!("Reply: {}", from_utf8(&data).unwrap());
                            }
                            Err(e) => {
                                println!("Failed to recieve data: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to recieve data: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
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
