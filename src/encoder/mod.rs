pub fn encode(string: &String) -> Vec<u8> {
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

pub fn decode(buf: &[u8]) -> String {
    let mut vec : Vec<u8> = Vec::with_capacity(buf.len());

    for b in buf {
        vec.push(decode_byte(*b));
    }

    String::from_utf8(vec).unwrap_or_else( |x| { println!("Error decoding. {:?}", x); String::from("") } )
}

fn decode_byte(b: u8) -> u8 {
    b ^ 0x11
}