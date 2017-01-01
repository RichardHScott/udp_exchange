extern crate uuid;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;

use uuid::Uuid;

use std::str::FromStr;
use std::net::{TcpListener, TcpStream, UdpSocket, Ipv4Addr, IpAddr, SocketAddr, SocketAddrV4};
use std::thread;

static USAGE: &'static str = "
Usage:
  udp_send server
  udp_send client <addr> <guid> <message>

Options:
    -h, --help   display this help and exit
    -v, --version   output version information and exit
";

#[derive(RustcDecodable)]
struct Args {
    cmd_client: bool,
    arg_addr: Option<String>,
    arg_guid: Option<String>,
    arg_message: Option<String>,
    cmd_server: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    if args.cmd_server {
        spawn_http_server();
        start_server();
    } else if args.cmd_client {
        let guid = Uuid::parse_str(&args.arg_guid.unwrap());

        match guid {
            Ok(id) => start_client(&args.arg_addr.unwrap(), &id, &args.arg_message.unwrap_or_else(| | panic!("Invalid message."))),
            ParseError => panic!("Parse error"),
        }
    }
}

fn start_client(address: &String, guid: &Uuid, msg: &String) {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        //let ip = IpAddr::V4(Ipv4Addr::from_str(address).unwrap());
        //let port: u16 = 5890;
        //let addr = SocketAddr::new(ip, port);
        let addr = SocketAddrV4::from_str(address).unwrap();

        let packet_message = create_message(guid, msg);

        socket.send_to(encode(&packet_message).as_slice(), addr);
    } else {
        println!("Error");
    }
}

fn create_message(uuid: &Uuid, msg: &String) -> String {
    let mut s = uuid.hyphenated().to_string();
    s.push_str(msg);
    s
}

fn start_server() -> ! {
    let socket = UdpSocket::bind("0.0.0.0:5890");

    match socket {
        Ok(x) => {
            println!("Socket connected");
            let mut clients = Clients::new();
            let buf = &mut [0; 1024];

            clients.add_client(Uuid::parse_str("deadbeef-dead-dead-dead-beefbeefbeef").unwrap());

            loop {
                let (amt, src) = x.recv_from(buf).unwrap();
                
                let decoded = decode(&buf[0..amt]);

                let (uuid, msg) = parse_packet_message(decoded.as_str());

                //println!("{:?}", &buf[0..amt]);
                //println!("amount: {} src: {:?} data: {} ", amt, src, decoded);
                //println!("uuid {:?} msg {:?}", uuid, msg.message);

                clients.add_message(uuid, msg);
            }
        },
        Err(x) => panic!("Error: {}", x)
    }
}

fn parse_packet_message(msg: &str) -> (Uuid, Data<String>) {
    (Uuid::parse_str(&msg[0..36]).unwrap(), Data { timestamp: String::from(""), message: String::from(&msg[36..]) })
}

fn encode(string: &String) -> Vec<u8> {
    let bytes = string.as_bytes();
    let mut encoded_bytes : Vec<u8> = Vec::with_capacity(bytes.len());

    for b in bytes {
        encoded_bytes.push(encode_byte(*b));
    }

    encoded_bytes
}

fn encode_byte(b: u8) -> u8 {
    b ^ 0x11
}

fn decode_byte(b: u8) -> u8 {
    b ^ 0x11
}

fn decode(buf: &[u8]) -> String {
    let mut vec : Vec<u8> = Vec::with_capacity(buf.len());
    //vec.extend_from_slice(buf);

    for b in buf {
        vec.push(decode_byte(*b));
    }

    String::from_utf8(vec).unwrap_or_else( |x| { println!("Error decoding. {:?}", x); String::from("") } )
}

struct Data<T>{
    timestamp: String,
    message: T,
}

impl<T> Data<T> {
    pub fn new(timestamp: String, msg: T) -> Data<T> {
        Data { timestamp: timestamp, message: msg }
    }
}

struct CircularList<T> {
    data: Vec<Option<T>>,
    current: usize,
}

impl<T> CircularList<T> {
    pub fn new(len: usize) -> CircularList<T> {
        let mut l = CircularList { data: Vec::with_capacity(len), current: 0 };
        for i in 1..len {
            l.data.push(None);
        }

        l
    }

    pub fn get(&self) -> Option<&T> {
        if let Some(ref x) = self.data[self.current] {
            Some(x)
        } else {
            None
        }
    }

    pub fn put(&mut self, data: T) {
        self.data[self.current] = Some(data);
        self.next();
    }

    pub fn next(&mut self) {
        self.current = self.current + 1;
        if self.current == self.data.len() {
            self.current = 0;
        }
    }
}

struct Client {
    name: String,
    guid: Uuid,
    list: CircularList<Data<String>>,
}

impl Client {
    fn new(guid: Uuid) -> Client {
        Client { name: String::from(""), guid: guid, list: CircularList::new(10) }
    }

    fn put_message(&mut self, data: Data<String>) {
        self.list.put(data);
    }
}

struct Clients {
    clients: Vec<Client>,
}

impl Clients {
    pub fn new() -> Clients {
        Clients { clients: Vec::new() }
    }

    pub fn add_client(&mut self, uuid: Uuid) {
        self.clients.push( Client::new(uuid) );
    }

    pub fn add_message(&mut self, uuid: Uuid, msg: Data<String>) {
        for client in &mut self.clients {
            if client.guid == uuid {
                client.put_message(msg);
                return;
            }
        }
    }
}

fn spawn_http_server() {


    thread::spawn(| | {
        let listener = TcpListener::bind("127.0.0.1:8787").unwrap();

        //Note this is single threaded.
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    serve(stream);
                },
                Err(e) => println!("Connection failed: {:?}", e)
            }
        }
    });
}

fn serve(mut stream :TcpStream) {
    use std::io::{Read, Write};
    use std::net::Shutdown;

    let ref mut buf: String = String::new();
    println!("Tcp stream connection from {:?}", stream);

    let buf = &mut [0; 1024];
    let num_read = stream.read(buf);

    let header = "HTTP/1.0 200 OK".as_bytes();
    let cr_lf ="\r\n".as_bytes();
    let cr_lf ="\r\n".as_bytes();
    let body = "<html><head></head><body><h1>test</h1></body></html>".as_bytes();

    stream.write_all(header);
    stream.write_all(cr_lf);
    stream.write_all(cr_lf);
    stream.write_all(body);
    stream.write_all(cr_lf);

    stream.shutdown(Shutdown::Both);
}