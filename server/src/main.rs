use std::io::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

pub struct Server {
    listener: TcpListener,
    header_size: usize,
    port: String,
}
impl Server {
    pub fn new(addr: &str, port: &str) -> Server {
        let listener = TcpListener::bind(String::from(addr) + ":" + port).unwrap();
        Server {
            listener,
            header_size: 8,
            port: String::from(port),
        }
    }
    pub fn listen(&mut self) {
        // accept connection and process them, spawning a new thread for each one
        println!("Server listening on port {}", self.port);
        for stream in self.listener.incoming() {
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
    }
}

pub struct Client {
    stream: TcpStream,
    header_size: usize,
}
impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client {
            stream,
            header_size: 8,
        }
    }
    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let mut head = vec![0 as u8; self.header_size];
        match self.stream.read(&mut head) {
            Ok(_) => {
                let len = from_utf8(&head).unwrap().parse::<usize>().unwrap();
                let mut data = vec![0 as u8; len];
                match self.stream.read_exact(&mut data) {
                    Ok(_) => Ok(data),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
    pub fn send(&mut self, msg: &[u8]) -> Result<usize, Error> {
        self.stream
            .write(pad_front(&msg.len().to_string(), HEADER_SIZE, "0").as_bytes())
            .unwrap();
        self.stream.write(msg)
    }
}

const HEADER_SIZE: usize = 8;

fn handle_client(stream: TcpStream) {
    let mut client = Client::new(stream);
    while match client.receive() {
        Ok(data) => {
            client.send(&data).unwrap();
            true
        }
        Err(e) => false,
    } {}
}

fn main() {
    let mut server = Server::new("127.0.0.1", "3333");
    server.listen();
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
