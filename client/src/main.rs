use std::io::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

const HEADER_SIZE: usize = 8;

pub struct Server {
    stream: TcpStream,
    header_size: usize,
}
impl Server {
    pub fn new(addr: &str, port: &str) -> Result<Server, Error> {
        match TcpStream::connect(String::from(addr) + ":" + port) {
            Ok(stream) => {
                println!("Successfully connected to server with port: {}", port);
                Ok(Server {
                    stream,
                    header_size: 8,
                })
            }
            Err(e) => Err(e),
        }
    }
    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let mut data = vec![0 as u8; self.header_size];
        match self.stream.read_exact(&mut data) {
            Ok(_) => {
                let len = from_utf8(&data).unwrap().parse::<usize>().unwrap();
                let mut data = vec![0 as u8; len]; // read header
                match self.stream.read_exact(&mut data) {
                    Ok(_) => {
                        // println!("Reply: {}", from_utf8(&data).unwrap());
                        Ok(data)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
    pub fn send(&mut self, msg: &[u8]) -> Result<usize, Error> {
        self.stream
            .write(pad_front(&msg.len().to_string(), self.header_size, "0").as_bytes())
            .unwrap();
        self.stream.write(&msg)
    }
}

fn main() {
    let mut server = Server::new("127.0.0.1", "3333").unwrap();
    loop {
        let mut msg = String::new();
        println!("Message: ");
        let _ = std::io::stdin().read_line(&mut msg).unwrap();
        let msg = msg.strip_suffix("\n").unwrap().as_bytes();
        server.send(msg).unwrap();
        println!("{}", from_utf8(&server.receive().unwrap()).unwrap());
    }
    println!("Terminated.");
}

fn pad_front(msg: &str, len: usize, ch: &str) -> String {
    let mut out = String::new();
    for _i in msg.len()..len {
        out += ch;
    }
    out + msg
}
fn pad_back(msg: &str, len: usize, ch: &str) -> String {
    let mut out = String::from(msg);
    for _i in msg.len()..len {
        out += ch;
    }
    out
}
