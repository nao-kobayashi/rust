extern crate time;
#[macro_use]
extern crate cfg_if;

use std::thread;
use std::net::{TcpListener,TcpStream};
use std::io::Read;

mod command;
mod result_code;
mod client;
use result_code::ResultCode;
use command::Command;
use client::*;

fn handle_client(mut stream: TcpStream) {
    println!("New client connected!");
    send_cmd(&mut stream, ResultCode::ServiceReadyForNewUser, "Welcome to this FTP server!");
    let mut client = Client::new(stream);
    loop {
        let data = read_all_message(&mut client.stream);
        if data.is_empty() {
            println!("client disconnected!");
            break;
        }
        client.handle_cmd(Command::new(data).unwrap());
    }
}


fn read_all_message(stream: &mut TcpStream) -> Vec<u8> {
    let buf = &mut [0; 1];
    let mut out = Vec::with_capacity(100);

    loop {
        match stream.read(buf) {
            Ok(received) if received > 0 => {
                if out.is_empty() && buf[0] == b' ' {
                    continue
                }
                out.push(buf[0]);
            }
            _ => return Vec::new(),
        }
        
        let len = out.len();
        if len > 1 && out[len - 2] == b'\r' && out[len - 1] == b'\n' {
            out.pop();
            out.pop();

            let converted = String::from_utf8(out.clone()).expect("debug error");
            println!("{}", converted);

            return out;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234").expect("Couldn't bind this address...");

    println!("Waiting for clients to connect...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            _ => {
                println!("A client tried to connect...");
            }
        }
    }
}
