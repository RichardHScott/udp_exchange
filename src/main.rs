extern crate uuid;
extern crate docopt;
extern crate rustc_serialize;
extern crate time;

use docopt::Docopt;

use uuid::Uuid;

use time::Timespec;

use std::str::FromStr;
use std::net::{TcpListener, TcpStream, UdpSocket, Ipv4Addr, IpAddr, SocketAddr, SocketAddrV4};
use std::thread;
use std::sync::{Arc, Mutex};

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
        let clients = Arc::new(Clients::new());

        spawn_http_server(clients.clone());
        start_server(clients.clone());
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

    let t = time::get_time();

    s.push_str(format!("({},{})", t.sec, t.nsec).as_str());

    s.push_str(msg);
    s
}

fn start_server(clients: Arc<Clients>) -> ! {
    let socket = UdpSocket::bind("0.0.0.0:5890");

    match socket {
        Ok(x) => {
            println!("Socket connected");
            //let mut clients = Clients::new();
            let buf = &mut [0; 1024];

            //clients.add_client(Uuid::parse_str("deadbeef-dead-dead-dead-beefbeefbeef").unwrap());

            loop {
                let (amt, src) = x.recv_from(buf).unwrap();
                
                let decoded = decode(&buf[0..amt]);

                if let Ok( (uuid, msg) ) = parse_packet_message(decoded.as_str()) {
                    clients.add_message(uuid, msg);
                } else {
                    println!("Error with message");
                }

                //println!("{:?}", &buf[0..amt]);
                //println!("amount: {} src: {:?} data: {} ", amt, src, decoded);
                //println!("uuid {:?} msg {:?}", uuid, msg.message);
            }
        },
        Err(x) => panic!("Error: {}", x)
    }
}

fn parse_packet_message(msg: &str) -> Result<(Uuid, Data<String>), &'static str> {
    if let Ok(data) = Data::<String>::new_from_message(&msg[36..]) {
        return Ok( (Uuid::parse_str(&msg[0..36]).unwrap(), data) );
    } else {
        return Err("Invalid message");
    }
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

#[derive(Clone)]
struct Data<T> {
    timestamp: Timespec,
    message: T,
}

impl<T : Clone + ToString> Data<T> {
    pub fn new_from_message(msg: &str) -> Result<Data<String>, &'static str> {
        if let Some(ts_end) = msg.find(')') {
            let timestamp_str = String::from(&msg[1..ts_end]);

            let sec: i64;
            let nsec: i32;

            if let Some(ts_mid) = timestamp_str.find(',') {
                sec = i64::from_str(&timestamp_str[0..ts_mid]).unwrap();
                nsec = i32::from_str(&timestamp_str[ts_mid+1..]).unwrap();
            } else {
                sec = 0;
                nsec = 0;
            }

            let ts = Timespec::new(sec, nsec);

            let message = String::from(&msg[ts_end+1..]);

            return Ok(Data { timestamp: ts, message: message });
        } else {
            return Err("Invalid message");
        }
    }
}

use std::cmp::Ordering;

impl<T: Clone + ToString> PartialEq for Data<T> {
    fn eq(&self, other: &Data<T>) -> bool {
                let lhs_ts = &self.timestamp;
        let rhs_ts = &other.timestamp;

        return lhs_ts.eq(rhs_ts);
    }

    fn ne(&self, other: &Data<T>) -> bool {
        let lhs_ts = &self.timestamp;
        let rhs_ts = &other.timestamp;

        return lhs_ts.ne(rhs_ts);
    }
}

impl<T: Clone + ToString> Eq for Data<T> { }

impl<T: Clone + ToString> PartialOrd for Data<T> {
    fn partial_cmp(&self, other: &Data<T>) -> Option<Ordering> {
        let lhs_ts = &self.timestamp;
        let rhs_ts = &other.timestamp;

        return lhs_ts.partial_cmp(rhs_ts);
    }
}

impl<T: Clone + ToString> ToString for Data<T> {
    fn to_string(&self) -> String {
        let secs = self.timestamp.sec.to_string();
        let nsecs = self.timestamp.nsec.to_string();

        let mut s = String::new();
        s.push_str(secs.as_str());
        s.push_str(nsecs.as_str());
        s.push_str(self.message.to_string().as_str());

        s
    }
}

impl<T: Clone + ToString> Ord for Data<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs_ts = &self.timestamp;
        let rhs_ts = &other.timestamp;

        return lhs_ts.cmp(rhs_ts);
    }
}

struct CircularList<T> {
    data: Vec<Option<T>>,
    current: usize,
}

impl<T : Clone> CircularList<T> {
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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_all(&self) -> Vec<T> {
        let mut vec = Vec::new();

        for v in &self.data {
            if let Some(ref x) = *v {
                vec.push(x.clone());
            }
        }

        vec
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

    fn get_name(&self) -> String {
        let mut result = self.name.clone();
        result.push_str(" :: ");
        result.push_str(self.guid.to_string().as_str());

        result
    }

    fn get_uuid(&self) -> Uuid {
        self.guid
    }

    fn get_messages(&self) -> Vec<String> {
        let vec = &mut self.list.get_all();
        vec.sort();

        let mut vec_str = Vec::with_capacity(vec.len());
        for v in vec {
            vec_str.push(v.to_string());
        }

        vec_str
    }
}

struct Clients {
    clients: Mutex<Vec<Client>>,
}

impl Clients {
    pub fn new() -> Clients {
        Clients { clients: Mutex::new(Vec::new()) }
    }

    pub fn add_client(&self, uuid: Uuid) {
        let vec = &mut self.clients.lock().unwrap();
        vec.push( Client::new(uuid) );
    }

    pub fn add_message(&self, uuid: Uuid, msg: Data<String>) {
        let mut vec = self.clients.lock().unwrap();

        for client in &mut *vec {
            if client.guid == uuid {
                client.put_message(msg);
                return;
            }
        }

        vec.push( Client::new(uuid) );
        vec.last_mut().unwrap().put_message(msg);
    }

    pub fn get_clients(&self) -> Vec<String> {
        let vec = self.clients.lock().unwrap();

        let mut res = Vec::with_capacity(vec.len());

        for client in & *vec {
            res.push( client.get_name() );
        }

        res
    }

    pub fn get_clients_uuids(&self) -> Vec<Uuid> {
        let vec = self.clients.lock().unwrap();

        let mut res = Vec::with_capacity(vec.len());

        for client in & *vec {
            res.push( client.get_uuid() );
        }

        res
    }

    pub fn get_messages_for_uuid(&self, uuid: &Uuid) -> Vec<String> {
        let vec = & *self.clients.lock().unwrap();

        for client in vec {
            if client.guid == *uuid {
                return client.get_messages();
            }
        }

        Vec::new()
    }
}

fn spawn_http_server(clients: Arc<Clients>) {
    thread::spawn(move | | {
        let listener = TcpListener::bind("127.0.0.1:8787").unwrap();

        //Note this is single threaded.
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    serve(stream, clients.clone());
                },
                Err(e) => println!("Connection failed: {:?}", e)
            }
        }
    });
}

fn serve(mut stream :TcpStream, clients: Arc<Clients>) {
    use std::io::{Read, Write};
    use std::net::Shutdown;

    let ref mut buf: String = String::new();
    println!("Tcp stream connection from {:?}", stream);

    //Currently ignoring request, serving static page
    let buf = &mut [0; 1024];
    let num_read = stream.read(buf);
    println!("Read {:?} bytes", num_read);

    //Response
    let header = "HTTP/1.0 200 OK".as_bytes();
    let cr_lf ="\r\n".as_bytes();

    let mut body_str = String::from("<html><head></head><body><h1>");
    body_str.push_str("test</h1>");

    body_str.push_str(format_vec_as_list(clients.get_clients()).as_str());

    for uuid in &clients.get_clients_uuids() {
        body_str.push_str(format_vec_as_list(clients.get_messages_for_uuid(uuid)).as_str());
    }

    body_str.push_str("</body></html>");

    let body = body_str.as_bytes();

    stream.write_all(header);
    stream.write_all(cr_lf);
    stream.write_all(cr_lf);
    stream.write_all(body);
    stream.write_all(cr_lf);

    stream.shutdown(Shutdown::Both);
}

fn format_vec_as_list(vec: Vec<String>) -> String {
    let mut result = String::with_capacity(vec.len()*80);

     String::from("<ul>");

    for s in &vec {
        result.push_str("<li>");
        result.push_str(s.as_str());
        result.push_str("</li>")
    }

    result.push_str("</ul>");
    result
}