use serde_bencode::{from_str, value::Value};
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]

fn decode(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::List(l) => format!(
            "[{}]",
            l.iter().map(decode).collect::<Vec<String>>().join(",")
        ),
        Value::Bytes(b) => format!("{:?}", String::from_utf8(b.clone()).unwrap()),
        Value::Dict(d) => {
            let mut result: Vec<String> = Vec::new();
            for (key, value) in d {
                let key_str = String::from_utf8_lossy(key).to_string();
                let val = decode(value);
                result.push(format!("\"{}\":{}", key_str, val));
            }
            result.sort();
            format!("{{{}}}", result.join(","))
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let value = from_str::<Value>(encoded_value).unwrap();
        println!("{}", decode(&value));
    } else {
        println!("unknown command: {}", args[1])
    }
}
