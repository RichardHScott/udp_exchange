extern crate uuid;
extern crate time;

use uuid::Uuid;

use std::sync::Mutex;
use std::net::SocketAddr;
use std::collections::HashMap;

mod data;
use clients::data::Data;

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

    fn get_messages(&self) -> Vec<(String, String)> {
        let vec = &mut self.list.get_all();
        vec.sort();

        let mut vec_str = Vec::with_capacity(vec.len());
        for v in vec {
            vec_str.push((v.get_time_as_string() , v.to_string()));
        }

        vec_str
    }
}

pub struct Clients {
    clients: Mutex<HashMap<Uuid, Client>>,
}

impl Clients {
    pub fn new() -> Clients {
        Clients { clients: Mutex::new(HashMap::new()) }
    }

    pub fn add_client(&self, uuid: Uuid) {
        let map = &mut self.clients.lock().unwrap();
        map.insert(uuid, Client::new(uuid) );
    }

    fn parse_packet_message(msg: String, src: SocketAddr) -> Result<(Uuid, Data<String>), &'static str> {
        if let Ok(data) = Data::<String>::new_from_message(&msg.as_str()[36..], src) {
            return Ok( (Uuid::parse_str(&msg[0..36]).unwrap(), data) );
        } else {
            return Err("Invalid message");
        }
    }

    pub fn add_message(&self, decoded: String, src: SocketAddr) -> Result<(), &'static str> {
        if let Ok( (uuid, msg) ) = Clients::parse_packet_message(decoded, src) {
            self.add_data_message(uuid, msg);
            Ok(())
        } else {
            Err("Unable to parse message")
        }
    }

    fn add_data_message(&self, uuid: Uuid, msg: Data<String>) {
        let mut map = self.clients.lock().unwrap();

        let entry = map.entry(uuid).or_insert( Client::new(uuid) );
        entry.put_message(msg);
    }

    pub fn get_clients(&self) -> Vec<String> {
        let map = self.clients.lock().unwrap();
        map.values().map(|v| { v.get_name() }).collect()
    }

    pub fn get_clients_uuids(&self) -> Vec<Uuid> {
        let map = self.clients.lock().unwrap();
        map.keys().map(|k| { k.clone() }).collect()
    }

    pub fn get_messages_for_uuid(&self, uuid: &Uuid) -> Vec<(String, String)> {
        let map = self.clients.lock().unwrap();

        match map.get(uuid) {
            Some(e) => e.get_messages(),
            None => Vec::new(),
        }
    }

    pub fn create_message(uuid: &Uuid, msg: &String) -> String {
        let mut s = uuid.hyphenated().to_string();

        let t = time::get_time();

        s.push_str(format!("({},{})", t.sec, t.nsec).as_str());
        s.push_str(msg);
        s
    }
}