extern crate uuid;
extern crate time;

use time::Timespec;
use uuid::Uuid;

use std::cmp::Ordering;
use std::str::FromStr;
use std::sync::Mutex;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Data<T> {
    timestamp: Timespec,
    source: SocketAddr,
    message: T,
}

impl<T : Clone + ToString> Data<T> {
    pub fn parse_packet_message(msg: String, src: SocketAddr) -> Result<(Uuid, Data<String>), &'static str> {
        if let Ok(data) = Data::<String>::new_from_message(&msg.as_str()[36..], src) {
            return Ok( (Uuid::parse_str(&msg[0..36]).unwrap(), data) );
        } else {
            return Err("Invalid message");
        }
    }

    fn new_from_message(msg: &str, src: SocketAddr) -> Result<Data<String>, &'static str> {
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

            return Ok(Data { timestamp: ts, message: message, source: src });
        } else {
            return Err("Invalid message");
        }
    }
}

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
    fn new(len: usize) -> CircularList<T> {
        let mut list = CircularList { data: Vec::with_capacity(len), current: 0 };
        for i in 0..len {
            list.data.push(None);
        }

        list
    }

    fn get(&self) -> Option<&T> {
        if let Some(ref x) = self.data[self.current] {
            Some(x)
        } else {
            None
        }
    }

    fn put(&mut self, data: T) {
        self.data[self.current] = Some(data);
        self.next();
    }

    fn next(&mut self) {
        self.current = self.current + 1;
        if self.current == self.data.len() {
            self.current = 0;
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get_all(&self) -> Vec<T> {
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

pub struct Clients {
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

    pub fn create_message(uuid: &Uuid, msg: &String) -> String {
        let mut s = uuid.hyphenated().to_string();

        let t = time::get_time();

        s.push_str(format!("({},{})", t.sec, t.nsec).as_str());
        s.push_str(msg);
        s
    }
}