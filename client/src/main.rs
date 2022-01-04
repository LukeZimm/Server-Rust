use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    let port = "3333";
    match TcpStream::connect(String::from("127.0.0.1:") + port) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port: {}", port);
            let msg = b"Hello!";
            stream.write(msg).unwrap();
            println!("Sent Hello, awaiting reply...");
            let mut data = [0 as u8; 6]; // 'Hello!' buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text)
                    }
                }
                Err(e) => {
                    println!("Failed to recieve data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
