use std;
use std::str;
use std::fs::read_dir;
use std::fs::Metadata;
use std::fs::metadata;
use std::path::Path;
use std::path::PathBuf;
use std::net::{TcpListener,TcpStream,IpAddr,Ipv4Addr,SocketAddr};
use std::io::Write;
use std::env::*;
use std::io::Error;
use time;

use result_code::ResultCode;
use command::Command;

pub struct Client {
    pub cwd: PathBuf,
    pub stream: TcpStream,
    pub name: Option<String>,
    pub data_writer: Option<TcpStream>,
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
            Command::List(path) | Command::Nlst(path) => {
                let server_root = current_dir().unwrap();
                let path = self.cwd.join(path);
                let directory = PathBuf::from(&path);
                let chk_path = &self.complete_path(directory, &server_root);

                if let Some(ref mut data_writer) = self.data_writer {
                    if let &Ok(ref path) = chk_path {
                        send_cmd(&mut self.stream, ResultCode::DataConnectionAlreadyOpen, "Starting to list directory...");
                        let mut out = String::new();
    
                        for entry in read_dir(path).unwrap() {
                            if let Ok(entry) = entry {
                                add_file_info(entry.path(), &mut out);
                            }
                            
                            send_data(data_writer, &out)
                        }
                    } else {
                        send_cmd(&mut self.stream, ResultCode::InvalidParameterOrArgument, "No such file or directory...");
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
                    let message = if msg == "/" {
                         format!("\"{}\" ", msg)
                    } else {
                         format!("\"/{}\" ", msg)
                    };
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
                    send_cmd(&mut self.stream, ResultCode::EnteringPassiveMode, &format!("127,0,0,1,{},{}", port >> 8, port & 0xFF));
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                    let listener = TcpListener::bind(&addr).unwrap();
                    match listener.incoming().next() {
                        Some(Ok(client)) => {
                            println!("{:?}", client);
                            self.data_writer = Some(client);
                        },
                        _ => {
                            send_cmd(&mut self.stream, ResultCode::ServiceNotAvailable, "issues happen");
                        }
                    }
                }
            },
            Command::Cwd(directory) => self.cwd(directory),
            Command::Cdup => {
                if let Some(path) = self.cwd.parent().map(Path::to_path_buf) {
                    self.cwd = path;
                }
                send_cmd(&mut self.stream, ResultCode::Ok, "Done");
            }
            Command::Unknown(_s) => send_cmd(&mut self.stream, ResultCode::UnknownCommand, "Not implemented"),
        }
    }

    fn complete_path(&self, path: PathBuf, server_root: &PathBuf) -> Result<PathBuf, Error> {
        let directory = server_root.join(if path.has_root(){
            path.iter().skip(1).collect()
        } else {
            path
        });

        let dir = directory.canonicalize();
        if let Ok(ref dir) = dir {
            if !dir.starts_with(&server_root) {
                return Err(std::io::ErrorKind::PermissionDenied.into());
            }
        }

        dir
    }

    fn cwd(&mut self, directory: PathBuf) {
        let server_root = current_dir().unwrap();
        let path = self.cwd.join(&directory);
        if let Ok(dir) = self.complete_path(path, &server_root) {
            if let Ok(prefix) = dir.strip_prefix(&server_root).map(|p| p.to_path_buf()) {
                self.cwd = prefix.to_path_buf();
                send_cmd(&mut self.stream, ResultCode::Ok, &format!("Directory changed to \"{}\"", directory.display()));
                return;
            }
        }
        send_cmd(&mut self.stream, ResultCode::FileNotFound, "No such file or directory");
    }
}

fn send_data(stream: &mut TcpStream, s: &str) {
    write!(stream, "{}", s).unwrap();
}

pub fn send_cmd(stream: &mut TcpStream, code: ResultCode, message: &str) {
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

    let meta = match metadata(&path) {
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


cfg_if! {
    if #[cfg(windows)] {
        fn get_fileinfo(meta: &Metadata) -> (time::Tm, u64) {
            use std::os::windows::prelude::*;
            (time::at(time::Timespec::new(meta.last_write_time() as i64, 0)), meta.file_size())
        }
    } else {
        fn get_fileinfo(meta: &Metadata) -> (time::Tm, u64) {
            use std::os::unix::prelude::*;
            (time::at(time::Timespec::new(meta.mtime(), 0)), meta.size())
        }
    }
}