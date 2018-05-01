use ini::Ini;
use std::str::FromStr;
use chrono::prelude::*;

pub struct ConfIni {
    pub ini_file: Ini,
}

fn get_ini(path: &String) -> Ini {
    let conf_path = get_ini_path(&path);
    Ini::load_from_file(conf_path).unwrap()
}

fn get_ini_path(path: &String) -> String {
    path.clone() + "\\conf.ini"
}

impl ConfIni {

    pub fn new(path: &String) -> ConfIni {
        let instance = ConfIni { ini_file: get_ini(path) };
        instance
    }

    pub fn get_ini_value(&self, key: String) -> String {
        let mut value = String::new();
        for (sec, prop) in self.ini_file.iter() {
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

    pub fn set_ini_modified_date(self, path: &String, date: DateTime<Local>) {
        let conf_path = get_ini_path(path);
        let mut ini_file = self.ini_file;
        ini_file.set_to(Some("kihon"), "lastmoddate".to_string(), date.to_string());
        match ini_file.write_to_file(&conf_path) {
            Err(_) => println!("conf.ini write error"),
            Ok(_) => println!("conf.ini write {:?}", date.to_string()),
        }
    }
}