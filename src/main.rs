use serde_bencode::{from_str, value::Value};
use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    let start = encoded_value.chars().next().unwrap();
    if start == 'l' {}
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "5"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return serde_json::Value::String(string.to_string());
    } else if encoded_value.chars().next().unwrap() == 'i' {
        let e_index = encoded_value.find('e').unwrap();
        let number = encoded_value[1..e_index].parse::<i64>().unwrap();
        return serde_json::Value::Number(number.into());
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

fn decode(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::List(l) => format!(
            "[{}]",
            l.iter()
                .map(|v| decode(v))
                .collect::<Vec<String>>()
                .join(",")
        ),
        Value::Bytes(b) => format!("{:?}", String::from_utf8(b.clone()).unwrap()),
        _ => panic!("Unhandled encoded value: {:?}", value),
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
        // let decoded_value = decode_bencoded_value(encoded_value);
        let value = from_str::<Value>(encoded_value).unwrap();
        // println!("{}", decoded_value.to_string());
        println!("{}", decode(&value));
    } else {
        println!("unknown command: {}", args[1])
    }
}
