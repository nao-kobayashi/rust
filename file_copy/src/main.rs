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

fn get_command_name(path: String) -> String {
    let mut command_name = String::new();
    let i = Ini::load_from_file(path + "\\conf.ini").unwrap();
    for (sec, prop) in i.iter() {
        if *sec == Some("kihon".to_string()) {
            for (k, v) in prop.iter() {
                if *k == "command" {
                     command_name = String::from_str(&*v.to_string()).expect("failed read ini file.");
                }
            }
        }
    }
    command_name
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
    let mut now: DateTime<Local> = std::time::SystemTime::now().into();
    now = now - chrono::Duration::days(1);
    println!("{:?}", now);

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
                    
                    //更新日が１日前以降が処理対象
                    if now < mod_date {
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
    
    let command_name = get_command_name(exe_path);
    //外部プログラム呼び出し
    Command::new(command_name)
        .arg(&arguments.path_to)
        .output().
        expect("failed to execute process");


    //コースを作成したのでコピーしたファイルを削除
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

