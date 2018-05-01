extern crate chrono;
extern crate ini;

use std::env;
use std::fs::*;
use std::path::PathBuf;
use chrono::prelude::*;
use std::process::Command;

mod conf_ini;

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


fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_path = match std::env::current_exe() {
            Ok(path) => match path.parent(){
                Some(parent) => parent.display().to_string(),
                None => panic!("Cannot get exe path."),
            },
            Err(e) => panic!("{:?}", e)
        };
    let inifile = conf_ini::ConfIni::new(&exe_path);

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
    let date = inifile.get_ini_value("lastmoddate".to_string());
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
    
    //外部プログラム呼び出し
    let command_name = inifile.get_ini_value("command".to_string());
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

    //最も新しかった日付を保存
    inifile.set_ini_modified_date(&exe_path, most_newest_date);

}
