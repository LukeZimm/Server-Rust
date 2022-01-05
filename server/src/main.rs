use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

pub struct Server {
    listener: TcpListener,
    header_size: usize,
    port: String,
    packet_types: HashMap<String, String>,
}
impl Server {
    pub fn new(addr: &str, port: &str) -> Server {
        let listener = TcpListener::bind(String::from(addr) + ":" + port).unwrap();
        let mut map = HashMap::new();
        map.insert(String::from("request"), String::from("rqst"));
        map.insert(String::from("data"), String::from("data"));
        map.insert(String::from("ping"), String::from("ping"));
        Server {
            listener,
            header_size: 8,
            port: String::from(port),
            packet_types: map,
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
    pub fn receive_basic(&mut self) -> Result<Vec<u8>, Error> {
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
    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let head_structure = self.head_structure();
        let mut header_size: usize = 0;
        for i in head_structure {
            header_size += i.1;
        }
        let mut data = vec![0 as u8; header_size];
        match self.stream.read(&mut data) {
            Ok(_) => {
                let mut head: HashMap<String, &[u8]> = HashMap::new();
                let mut current: usize = 0;
                for i in self.head_structure() {
                    head.insert(String::from(i.0), &data[current..current + i.1]);
                    current += i.1
                }
                let len = head.get("len").copied().unwrap();
                let len = usize::from_le_bytes(pop(len));
                let pkg_type = from_utf8(head.get("pkg_type").unwrap()).unwrap();
                let mut data = vec![0 as u8; len];
                match self.stream.read_exact(&mut data) {
                    Ok(_) => {
                        match pkg_type {
                            "data" => {
                                println!("{}", from_utf8(&data).unwrap())
                            }
                            "ping" => {
                                self.send(b"ping", b"ping").unwrap();
                                println!("Client: {} pinged", self.stream.peer_addr().unwrap())
                            }
                            &_ => {}
                        }
                        Ok(data)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
    pub fn send_basic(&mut self, msg: &[u8]) -> Result<usize, Error> {
        self.stream
            .write(pad_front(&msg.len().to_string(), HEADER_SIZE, "0").as_bytes())
            .unwrap();
        self.stream.write(msg)
    }
    pub fn send(&mut self, pkg_type: &[u8], msg: &[u8]) -> Result<usize, Error> {
        let len = msg.len().to_le_bytes();
        let mut head = Vec::from(len);
        head.extend_from_slice(pkg_type);
        // println!("{:?}", head);
        self.stream.write(&head).unwrap();
        self.stream.write(msg)
    }
    pub fn handshake_data(&mut self) -> Vec<u8> {
        let len_length: usize = 8;
        let pkg_type_length = "ping".as_bytes().len();
        let head_structure = [("len", len_length), ("pkg_type", pkg_type_length)];
        let mut string = String::new();
        for tuple in head_structure {
            string += &format!("{}:{},", tuple.0, tuple.1);
        }
        string = String::from(&string[..string.len() - 1]); // remove trailing ,
                                                            // println!("{}", string);
        string.into_bytes()
    }
    pub fn head_structure(&mut self) -> Vec<(&str, usize)> {
        let len_length: usize = 8;
        let pkg_type_length = "ping".as_bytes().len();
        vec![("len", len_length), ("pkg_type", pkg_type_length)]
    }
    pub fn handshake(&mut self) {
        /* let num = (51234 as u32).to_le_bytes();
        self.send_basic(&num).unwrap(); */
        let ping = self.receive_basic().unwrap();
        self.send_basic(&ping).unwrap();
        let i = self.handshake_data();
        self.send_basic(&i).unwrap();
        self.send(b"rqst", b"name").unwrap(); // change
        let name = self.receive().unwrap();
        // println!("name: {}", from_utf8(&name).unwrap());
    }
}

const HEADER_SIZE: usize = 8;
// const HEAD: Vec<(String, usize)> = vec![(String::from("len"), 4), (String::from("cmd"), 4)];

fn handle_client(stream: TcpStream) {
    let mut client = Client::new(stream);
    client.handshake();

    loop {
        client.receive().unwrap();
    }
}

fn main() {
    let mut server = Server::new("0.0.0.0", "3333");
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

fn pop(barry: &[u8]) -> [u8; 8] {
    barry.try_into().expect("slice with incorrect length")
}
