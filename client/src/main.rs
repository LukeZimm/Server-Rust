use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::time::{Duration, Instant};

const HEADER_SIZE: usize = 8;

pub struct Server {
    stream: TcpStream,
    header_size: usize,
    handshake_data: Vec<(String, usize)>,
}
impl Server {
    pub fn new(addr: &str, port: &str) -> Result<Server, Error> {
        match TcpStream::connect(String::from(addr) + ":" + port) {
            Ok(stream) => {
                println!("Successfully connected to server with port: {}", port);
                Ok(Server {
                    stream,
                    header_size: 8,
                    handshake_data: Vec::new(),
                })
            }
            Err(e) => Err(e),
        }
    }
    pub fn receive_basic(&mut self) -> Result<Vec<u8>, Error> {
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
    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let mut data = vec![0 as u8; self.header_size];
        match self.stream.read_exact(&mut data) {
            Ok(_) => {
                // println!("{:?}", data);
                let mut head: HashMap<String, &[u8]> = HashMap::new();
                let mut current: usize = 0;
                for i in &self.handshake_data {
                    head.insert(String::from(&i.0), &data[current..current + i.1]);
                    current += i.1;
                }
                // println!("{:?}", head);
                let len = head.get("len").copied().unwrap();
                let len = usize::from_le_bytes(pop(len));
                let pkg_type = from_utf8(head.get("pkg_type").unwrap()).unwrap();
                let mut data = vec![0 as u8; len]; // read header
                match self.stream.read_exact(&mut data) {
                    Ok(_) => {
                        match pkg_type {
                            "rqst" => {
                                let request = from_utf8(&data).unwrap();
                                match request {
                                    "name" => {
                                        let mut msg = String::new();
                                        println!("Name: ");
                                        let _ = std::io::stdin().read_line(&mut msg).unwrap();
                                        let msg = msg.strip_suffix("\n").unwrap().as_bytes();
                                        self.send(b"data", msg).unwrap();
                                    }
                                    &_ => {}
                                }
                            }
                            "ping" => {}
                            "data" => {}
                            &_ => {}
                        }
                        // println!("Reply: {}", from_utf8(&data).unwrap());
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
            .write(pad_front(&msg.len().to_string(), 8, "0").as_bytes()) // change hard coded 8
            .unwrap();
        self.stream.write(&msg)
    }
    pub fn send(&mut self, pkg_type: &[u8], msg: &[u8]) -> Result<usize, Error> {
        let len = msg.len().to_le_bytes();
        let mut head = Vec::from(len);
        head.extend_from_slice(pkg_type);
        self.stream.write(&head).unwrap();
        self.stream.write(msg)
    }
    pub fn handshake(&mut self) {
        println!("ping: {}ms", self.ping_basic().as_millis());
        let data = self.receive_basic().unwrap();
        let data = from_utf8(&data).unwrap();
        // println!("{}", data);
        let data = data.split(",");
        let mut handshake_data: Vec<(String, usize)> = Vec::new();
        let mut header_size: usize = 0;
        for i in data {
            let j: Vec<&str> = i.split(":").collect();
            let k = j[1].parse::<usize>().unwrap();
            handshake_data.push((String::from(j[0]), k));
            header_size += k;
        }
        self.handshake_data = handshake_data;
        self.header_size = header_size;
    }
    pub fn ping_basic(&mut self) -> Duration {
        let t1 = Instant::now();
        self.send_basic(b"ping").unwrap();
        self.receive_basic().unwrap();
        let t2 = Instant::now();
        t2 - t1
    }
    pub fn ping(&mut self) -> Duration {
        let t1 = Instant::now();
        self.send(b"ping", b"ping").unwrap();
        self.receive().unwrap();
        let t2 = Instant::now();
        t2 - t1
    }
}

fn main() {
    let mut server = Server::new("127.0.0.1", "3333").unwrap();
    // get handshake information
    server.handshake();
    // header length
    // commands
    server.receive().unwrap();
    println!("ping {}ms", server.ping().as_millis());
    /* loop {
        let mut msg = String::new();
        println!("Message: ");
        let _ = std::io::stdin().read_line(&mut msg).unwrap();
        let msg = msg.strip_suffix("\n").unwrap().as_bytes();
        server.send(msg).unwrap();
        println!("{}", from_utf8(&server.receive().unwrap()).unwrap());
    } */
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

fn pop(barry: &[u8]) -> [u8; 8] {
    barry.try_into().expect("slice with incorrect length")
}
