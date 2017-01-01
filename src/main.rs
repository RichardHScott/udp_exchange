extern crate uuid;
extern crate docopt;
extern crate rustc_serialize;
extern crate time;

use docopt::Docopt;

use uuid::Uuid;

use std::str::FromStr;
use std::net::{UdpSocket, SocketAddr, SocketAddrV4};
use std::sync::Arc;

mod clients;
use clients::Clients;

mod http_server;
use http_server::*;

mod encoder;
use encoder::*;

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

        http_server::spawn_http_server(clients.clone());
        start_server(clients.clone());
    } else if args.cmd_client {
        let guid = Uuid::parse_str(&args.arg_guid.unwrap());

        match guid {
            Ok(id) => start_client(&args.arg_addr.unwrap(), &id, &args.arg_message.unwrap_or_else(| | panic!("Invalid message."))),
            _ => panic!("Parse error"),
        }
    }

    return;
}

fn start_client(address: &String, guid: &Uuid, msg: &String) {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        let addr = SocketAddrV4::from_str(address).unwrap();

        let packet_message = Clients::create_message(guid, msg);

        match socket.send_to(encode(&packet_message).as_slice(), addr) {
            Ok(_) => return,
            Err(_) => panic!("Error sending message"),
        }
    } else {
        panic!("Error opening socket");
    }
}

fn start_server(clients: Arc<Clients>) -> ! {
    let socket = UdpSocket::bind("0.0.0.0:5890");

    match socket {
        Ok(x) => {
            let buf = &mut [0; 2048];
            loop {
                let amt: usize;
                let src: SocketAddr;

                if let Ok( (a,s) ) = x.recv_from(buf) {
                    amt = a;
                    src = s;
                } else {
                    println!("Error receiving packet!");
                    continue;
                }
                
                let decoded = decode(&buf[0..amt]);

                if let Ok( () ) = clients.add_message(decoded, src) {
                } else {
                    println!("Error with message");
                }
            }
        },
        Err(x) => panic!("Error: {}", x)
    }
}