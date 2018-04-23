use std::str;
use std::fs::read_dir;
use std::path::PathBuf;
use std::thread;
use std::net::{TcpListener,TcpStream,IpAddr,Ipv4Addr,SocketAddr};
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
    data_writer: Option<TcpStream>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client {
            cwd: PathBuf::from("/"),
            stream: stream,
            name: None,
            data_writer: None,
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
            Command::List => {
                if let Some(ref mut data_writer) = self.data_writer {
                    let mut tmp = PathBuf::from(".");
                    send_cmd(&mut self.stream, ResultCode::DataConnectionAlreadyOpen, "Starting to list directory...");
                    let mut out = String::new();
                    for entry in read_dir(tmp).unwrap() {
                        if let Ok(entry) = entry {
                            add_file_info(entry.path(), &mut out);
                        }
                        send_data(data_writer, &out)
                    }
                } else {
                    send_cmd(&mut self.stream, ResultCode::ConnectionClosed, "No opened data connection");
                }

                if self.data_writer.is_some() {
                    self.data_writer = None;
                    send_cmd(&mut self.stream, ResultCode::ClosingDataConnection, "Transfer Done...");
                }
            },
            Command::NoOp => send_cmd(&mut self.stream, ResultCode::Ok, "Doing nothing..."),
            Command::Pwd | Command::XPwd => {
                let msg = format!("{}", self.cwd.to_str().unwrap_or(""));
                if !msg.is_empty() {
                    let message;
                    if msg == "/" {
                         message = format!("\"{}\" ", msg);
                    } else {
                         message = format!("\"/{}\" ", msg);
                    }
                    send_cmd(&mut self.stream, ResultCode::PATHNAMECreated, &message)
                } else {
                    send_cmd(&mut self.stream, ResultCode::FileNotFound, "No such file or directory")
                }
            },
            Command::Pasv => {
                if self.data_writer.is_some() {
                    send_cmd(&mut self.stream, ResultCode::DataConnectionAlreadyOpen, "Already listening...")
                } else {
                    let port = 43210;
                    send_cmd(&mut self.stream, ResultCode::EnteringPassiveMode, &format!("127.0.0.1 {} {}", port >> 8, port & 0xFF));
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                    let listener = TcpListener::bind(&addr).unwrap();
                    match listener.incoming().next() {
                        Some(Ok(client)) => {
                            self.data_writer = Some(client);
                        },
                        _ => {
                            send_cmd(&mut self.stream, ResultCode::ServiceNotAvailable, "issues happen");
                        }
                    }
                }
            },
            Command::Cwd(_path) => send_cmd(&mut self.stream, ResultCode::CommandNotImplemented, "Not implemented"),
            Command::Unknown(_s) => send_cmd(&mut self.stream, ResultCode::UnknownCommand, "Not implemented"),
        }
    }
}

fn send_data(stream: &mut TcpStream, s: &str) {
    write!(stream, "{}", s).unwrap();
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

fn add_file_info(path: PathBuf, out: &mut String) {
    let extra = if path.is_dir() {"/"} else {""};
    let is_dir = if path.is_dir() {"d"} else {"-"};

    let meta = match std::fs::metadata(&path) {
        Ok(meta) => meta,
        _ => return,
    };

    let (time, file_size) = get_fileinfo(&meta);
    let path = match path.to_str() {
        Some(path) => match path.split("/").last() {
            Some(path) => path,
            _ => return,
        },
        _ => return,
    };

    let rights = if meta.permissions().readonly() {
        "r--r--r--"
    } else {
        "rw-rw-rw-"
    };

    let file_str = format!("{is_dir}{rights} {links} {owner} {group} {size} {month} 
        {day} {hour}:{min} {path}{extra}\r\n",
        is_dir=is_dir,
        rights=rights,
        links=1, // number of links
        owner="anonymous", // owner name
        group="anonymous", // group name
        size=file_size,
        month=time.tm_mon as usize,
        day=time.tm_mday,
        hour=time.tm_hour,
        min=time.tm_min,
        path=path,
        extra=extra);

    out.push_str(&file_str);
    println!("==>{:?}", &file_str);
}