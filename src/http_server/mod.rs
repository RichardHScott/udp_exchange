use super::clients::Clients;

use std::sync::Arc;
use std::thread;
use std::net::{TcpStream, TcpListener};

pub struct http_server {
}

impl http_server {
    pub fn spawn_http_server(clients: Arc<Clients>) {
        thread::spawn(move | | {
            let listener = TcpListener::bind("127.0.0.1:8787").unwrap();

            //Note this is single threaded.
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        use std::time::Duration;
                        
                        stream.set_read_timeout(Some(Duration::from_millis(250))).ok();
                        
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

        println!("Tcp stream connection from {:?}", stream);

        //Currently ignoring request, serving static page
        let buf = &mut [0; 1024];
        let num_read = stream.read(buf);
        println!("Read {:?} bytes", num_read);

        //Response
        let header = "HTTP/1.0 200 OK".as_bytes();
        let cr_lf = "\r\n".as_bytes();

        let mut body_str = String::from("<html><head></head><body><h1>");
        body_str.push_str("test</h1>");

        body_str.push_str(format_vec_as_list(clients.get_clients()).as_str());

        for uuid in &clients.get_clients_uuids() {
            body_str.push_str(format_vec_as_list(clients.get_messages_for_uuid(uuid)).as_str());
        }

        body_str.push_str("</body></html>");

        let body = body_str.as_bytes();

        stream.write_all(header)
            .and_then(|_| { stream.write_all(cr_lf) } )
            .and_then(|_| { stream.write_all(cr_lf) } )
            .and_then(|_| { stream.write_all(body) } )
            .and_then(|_| { stream.write_all(cr_lf) } ).ok();

        stream.shutdown(Shutdown::Both).ok();
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
}