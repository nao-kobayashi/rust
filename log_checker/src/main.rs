extern crate chrono;
extern crate ini;
extern crate regex;

use std::env;
use std::path::PathBuf;
use std::process;
use log_file::LogFile;
use chrono::prelude::*;

mod conf_ini;
mod log_file;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("パラメータの数が違います。ex) log_checker <logfile full path> <regex str>");
        process::exit(0);
    }

    //指定されたログファイルが存在するか
    let logfilepath = PathBuf::from(&args[1]);
    if !logfilepath.is_file() {
        println!("ログファイルが存在しません。");
        process::exit(0);
    }

    //INIファイル参照
    let exe_path = match std::env::current_exe() {
            Ok(path) => match path.parent(){
                Some(parent) => parent.display().to_string(),
                None => panic!("Cannot get exe path."),
            },
            Err(e) => panic!("{:?}", e)
        };
    let mut inifile = conf_ini::ConfIni::new(&exe_path);

    //前回読んだログファイルの最終更新日を取得
    let mod_date = match inifile.get_ini_value("lastmoddate".to_string()) {
        Some(mod_date) => {
            Local.datetime_from_str(&mod_date, "%Y-%m-%d %H:%M:%S%.6f %z").unwrap()
        },
        None => {
            let mut now: DateTime<Local> = std::time::SystemTime::now().into();
            now = now - chrono::Duration::days(365);
            now
        }
    };

    //前回読んだログファイルの最終行を取得
    let line = match inifile.get_ini_value("readline".to_string()) {
        Some(line) => {
            line.parse::<u32>().unwrap()
        },
        None => {
            0
        }
    };

    //ログファイルのチェック
    let mut log_file = LogFile::new(logfilepath.to_str().unwrap().to_string(), mod_date, line, args[2].clone());
    let result_line = match log_file.find_error() {
        Ok(line) => line,
        Err(e) => {
            println!("{:?}", &e[..]);
            process::exit(0);
        }
    };

    //結果をiniファイルに書き込み
    let mut collection = conf_ini::IniCollection::new();
    collection.add("lastmoddate".to_string(), log_file.get_log_mod_date());
    collection.add("readline".to_string(), result_line.to_string());

    match inifile.set_ini_value(&exe_path, collection) {
        Ok(_) => {},
        Err(e) => {
            println!("{:?}", &e[..]);
            process::exit(0);
        }
    };

}
