extern crate chrono;
extern crate ini;

use std::env;
use std::fs::*;
use std::path::PathBuf;
use chrono::prelude::*;
use std::process::Command;
use std::str::FromStr;
use ini::Ini;

struct Arguments {
    path_from: PathBuf,
    path_to: PathBuf
}

fn check_arguments(args: &Vec<String>) -> Result<Arguments, String> {

    if args.len() != 4 {
        return Err("Parameter count is wrong.".to_owned());
    }

    if args[1] != "batch" {
        return Err("Invalid parameter".to_owned());
    }

    let dir = &args[2];
    let path_from = PathBuf::from(&dir);
    if !path_from.is_dir() {
        return Err("Not found source directory.".to_owned());
    }

    let dir_to = &args[3];
    let path_to = PathBuf::from(&dir_to);
    if !path_to.is_dir() {
        return Err("Not found destination directory.".to_owned());
    }

    let arguments = Arguments {
        path_from: path_from,
        path_to: path_to
    };

    Ok(arguments)
}

fn get_ini(path: &String) -> Ini {
    let ini_file = Ini::load_from_file(get_ini_path(path)).unwrap();
    ini_file
}

fn get_ini_path(path: &String) -> String {
    path.clone() + "\\conf.ini"
}

fn get_ini_value(path: String, key: String) -> String {
    let mut value = String::new();
    let ini_file = get_ini(&path);
    for (sec, prop) in ini_file.iter() {
        if *sec == Some("kihon".to_string()) {
            for (k, v) in prop.iter() {
                if *k == key {
                     value = String::from_str(&*v.to_string()).expect("failed read ini file.");
                }
            }
        }
    }
    value
}

fn set_ini_modified_date(path: String, date: DateTime<Local>) {
    let conf_path = get_ini_path(&path);
    let mut ini_file = get_ini(&path);
    ini_file.set_to(Some("kihon"), "lastmoddate".to_string(), date.to_string());
    match ini_file.write_to_file(&conf_path) {
        Err(_) => println!("conf.ini write error"),
        Ok(_) => println!("conf.ini write {:?}", date.to_string()),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_path = match std::env::current_exe() {
            Ok(path) => match path.parent(){
                Some(parent) => parent.display().to_string(),
                None => panic!("Cannot get exe path."),
            },
            Err(e) => panic!("{:?}", e)
        };

    //パラメータのチェック
    let mut arguments = match check_arguments(&args) {
        Ok(arguments) => {
            arguments
        },
        Err(msg) => {
            println!("{:?}", msg);
            return;
        }
    };

    //1日前をセット
    let date = get_ini_value(exe_path.clone(), "lastmoddate".to_string());
    let last_update_date: DateTime<Local> = match DateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S%.6f %z") {
        Ok(date) => {
            Local.datetime_from_str(&date.to_string(), "%Y-%m-%d %H:%M:%S%.6f %z").unwrap()
        },
        Err(e) => {
            println!("datetime parse error {:?}", e);
            let mut now = std::time::SystemTime::now().into();
            now = now - chrono::Duration::days(1);
            now
        },
    };

    println!("target date is over {:?}", last_update_date);
    let mut most_newest_date = last_update_date.clone();

    //取込対象ファイルをコピー
    match read_dir(arguments.path_from) {
        Err(err) => println!("{:?}", err.kind()),
        Ok(paths) => {
            for path in paths {
                let p = path.unwrap().path();
                if p.is_file() {
                    let metadata = metadata(&p) ;
                    let mod_date: DateTime<Local> = match metadata {
                        Ok(metadata) => {
                            metadata.modified().unwrap().into()
                        },
                        Err(e) => panic!("file property not read. {:?}", e)
                    };
                    
                    //前回に処理した最終更新日より新しいファイルを対象とする
                    if last_update_date < mod_date {
                        //より新しい日付を保存する。
                        if most_newest_date < mod_date {
                            most_newest_date = mod_date;
                        }

                        println!("copy target {:?}", &p);
                        arguments.path_to.push(&p.file_name().unwrap());
                        println!("copy dest {:?}", &arguments.path_to.as_path());
                        match copy(&p, &arguments.path_to.as_path()) {
                            Ok(ret_cd) => {
                                println!("Copy OK {}KB", ret_cd as f32 / 1024 as f32)
                            },
                            _ => println!("Copy NG"),
                        }
                        arguments.path_to.pop();
                    }
                }
            }
        },
    }
    

    //最も新しかった日付を保存
    set_ini_modified_date(exe_path.clone(), most_newest_date);

    //外部プログラム呼び出し
    let command_name = get_ini_value(exe_path.clone(), "command".to_string());
    if !command_name.is_empty() {
        Command::new(command_name)
            .arg(&arguments.path_to)
            .output().
            expect("failed to execute process");

        //コピーしたファイルを削除
        match read_dir(arguments.path_to) {
            Err(e) => println!("{:?}", e.kind()),
            Ok(paths) => {
                for path in paths {
                    let p = path.unwrap().path();
                    if p.is_file() {
                        match remove_file(&p) {
                            Ok(()) => println!("deleted file at {:?}", &p),
                            Err(_) => println!("fail delete file {:?}", &p),
                        }
                    }
                }
            }
        }
    }
}

