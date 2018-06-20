extern crate chrono;
extern crate ini;

use std::env;
use std::fs::*;
use std::path::PathBuf;
use chrono::prelude::*;
use std::process::Command;
use std::thread;
use std::sync::mpsc;

mod conf_ini;

struct Arguments<'a> {
    path_from: Box<&'a str>,
    path_to: Box<&'a str>
}

fn check_arguments(args: Vec<String>) -> Result<(PathBuf, PathBuf), String> {

    if args.len() != 3 {
        return Err("Parameter count is wrong.".to_owned());
    }

    let dir = &args[1];
    let path_from: PathBuf = PathBuf::from(&dir);
    if !path_from.is_dir() {
        return Err("Not found source directory.".to_owned());
    }

    let dir_to = &args[2];
    let path_to = PathBuf::from(&dir_to);
    if !path_to.is_dir() {
        return Err("Not found destination directory.".to_owned());
    }
    
    Ok((path_from, path_to))
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
    let param_pathes = match check_arguments(args) {
        Ok(arguments) => {
            arguments
        },
        Err(msg) => {
            println!("{:?}", msg);
            return;
        }
    };

    let arguments = Arguments {
        path_from: Box::new(param_pathes.0.to_str().unwrap()),
        path_to: Box::new(param_pathes.1.to_str().unwrap())
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

    let paths = match read_dir(PathBuf::from(arguments.path_from.as_ref().to_string())) {
        Err(err) => {
            println!("{:?}", err.kind());
            std::process::exit(0);
        }
        Ok(paths) => paths
    };

    let mut path_count = 0;
    let (tx, rx) = mpsc::channel();
    let handles: Vec<_>  = paths.map(|p| {
        let tx_clone = tx.clone();
        let p_str = p.unwrap().path().to_str().unwrap().to_owned();
        let mut path_to = PathBuf::from(arguments.path_to.as_ref().to_string());
        path_count = path_count + 1;
        thread::spawn(move || {
            let path_closure = PathBuf::from(p_str);
            if path_closure.is_file() {
                let metadata = metadata(&path_closure) ;
                let mod_date: DateTime<Local> = match metadata {
                    Ok(metadata) => {
                        metadata.modified().unwrap().into()
                    },
                    Err(e) => panic!("file property not read. {:?}", e)
                };
                
                //前回に処理した最終更新日より新しいファイルを対象とする
                if last_update_date < mod_date {
                    println!("copy target {:?}", &path_closure);
                    path_to.push(&path_closure.file_name().unwrap());
                    println!("copy dest {:?}", &path_to.as_path());
                    match copy(&path_closure, &path_to.as_path()) {
                        Ok(ret_cd) => {
                            println!("Copy OK {}KB", ret_cd as f32 / 1024 as f32)
                        },
                        _ => println!("Copy NG"),
                    }
                    path_to.pop();
                }
                tx_clone.send(mod_date).unwrap();
            } else {
                tx_clone.send(last_update_date).unwrap();
            }
        })
    }).collect();


    //全部のスレッドが終了するまで待つ
    for h in handles {
        h.join().unwrap();
        let temp_date = rx.recv().unwrap();
        if most_newest_date < temp_date {
            most_newest_date = temp_date;
        }
    }

    
    //外部プログラム呼び出し
    let command_name = inifile.get_ini_value("command".to_string());
    if !command_name.is_empty() {
        Command::new(command_name)
            .arg(arguments.path_to.as_ref())
            .output().
            expect("failed to execute process");

        //コピーしたファイルを削除
        match read_dir(arguments.path_to.as_ref().to_string()) {
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
