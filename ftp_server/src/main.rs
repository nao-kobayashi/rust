use std::str;
use std::path::PathBuf;
use std::thread;
use std::net::{TcpListener,TcpStream};
use std::io::{Read, Write};

mod result_code;
use result_code::ResultCode;

mod command;
use command::Command;


pub fn handle_client(mut stream: TcpStream) {
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




struct Client {
    cwd: PathBuf,
    stream: TcpStream,
    name: Option<String>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client {
            cwd: PathBuf::from("/"),
            stream: stream,
            name: None,
        }
    }

    pub fn handle_cmd(&mut self, cmd: Command) {
        println!("=====>{:?}", cmd);
        match cmd {
            Command::Auth => send_cmd(&mut self.stream, ResultCode::CommandNotImplemented, "Not implemented"),
            Command::Syst => send_cmd(&mut self.stream, ResultCode::Ok, "I won't tell"),
            Command::User(username) => {
                if username.is_empty() {
                    send_cmd(&mut self.stream, ResultCode::InvalidParameterOrArgument, "Invalid username")
                } else {
                    self.name = Some(username.to_owned());
                    send_cmd(&mut self.stream, ResultCode::UserLoggedIn, &format!("Welcome {}!", username))
                }
            },
            Command::NoOp => send_cmd(&mut self.stream, ResultCode::Ok, "Doing nothing..."),
            Command::Pwd => {
                let msg = format!("{}", self.cwd.to_str().unwrap_or(""));
                if !msg.is_empty() {
                    let message = format!("\"/{}\" ", msg);
                    send_cmd(&mut self.stream, ResultCode::PATHNAMECreated, &message)
                } else {
                    send_cmd(&mut self.stream, ResultCode::FileNotFound, "No such file or directory")
                }
            },
            Command::Cwd(_path) => send_cmd(&mut self.stream, ResultCode::CommandNotImplemented, "Not implemented"),
            Command::Unknown(_s) => send_cmd(&mut self.stream, ResultCode::UnknownCommand, "Not implemented"),
        }
    }
}

fn send_cmd(stream: &mut TcpStream, code: result_code::ResultCode, message: &str) {
    let msg = if message.is_empty() {
        format!("{}\r\n", code as u32)
    } else {
        format!("{} {}\r\n", code as u32, message)
    };

    println!("<====={}", msg);
    write!(stream, "{}", msg).unwrap()
}