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
    pub fn new_from_message(msg: &str, src: SocketAddr) -> Result<Data<String>, &'static str> {
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