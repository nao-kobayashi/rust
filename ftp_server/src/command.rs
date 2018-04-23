use std::io;
use std::str;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Command {
    Auth,
    Syst,
    User(String),
    List,
    NoOp,
    Pwd,
    XPwd,
    Pasv,
    Cwd(PathBuf),
    Unknown(String),
}

impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match *self {
            Command::Auth => "AUTH",
            Command::Syst => "SYST",
            Command::User(_) => "USER",
            Command::List => "LIST",
            Command::NoOp => "NOOP",
            Command::Pwd => "PWD",
            Command::XPwd => "XPWD",
            Command::Pasv => "PASV",
            Command::Cwd(_) => "CWD",
            Command::Unknown(_) => "UNKN",
        }
    }
}

impl Command {
    pub fn new(input: Vec<u8>) -> io::Result<Self> {
        let mut iter = input.split(|&byte| byte == b' ');
        let mut command = iter.next().expect("command in input").to_vec();
        to_uppercase(&mut command);
        let data = iter.next();
        let command = 
            match command.as_slice() {
                b"AUTH" => Command::Auth,
                b"SYST" => Command::Syst,
                b"LIST" => Command::List,
                b"NOOP" => Command::NoOp,
                b"PWD" => Command::Pwd,
                b"XPWD" => Command::XPwd,
                b"PASV" => Command::Pasv,
                b"USER" => Command::User(data.map(|bytes|String::from_utf8(bytes.to_vec()).expect("cannot convert bytes to String")).unwrap_or_default()),
                b"CWD" => Command::Cwd(data.map(|bytes| Path::new(str::from_utf8(bytes).unwrap()).to_path_buf()).unwrap()),
                s => Command::Unknown(str::from_utf8(s).unwrap_or("").to_owned()),
            };
        Ok(command)
    }
}

fn to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte >= 'a' as u8 && *byte <= 'z' as u8 {
            *byte -= 32;
        }
    }
}