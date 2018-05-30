use ini::Ini;
use std::str::FromStr;

pub struct ConfIni {
    pub ini_file: Ini,
}

pub struct IniCollection {
    keys: Vec<String>,
    values: Vec<String>,
    count: usize,
    current: usize,
}

impl IniCollection {
    pub fn new() -> IniCollection {
        IniCollection {
            keys: vec![],
            values: vec![],
            count: 0,
            current: 0,
        }
    }

    pub fn add(&mut self, key: String, value: String){
        self.keys.push(key);
        self.values.push(value);
        self.count += 1;
    }
}

impl Iterator for IniCollection {
    type Item = (String, String);

    fn next(&mut self) -> Option<(String, String)> {
        if self.count <= self.current {
            None
        } else {
            self.current += 1;
            Some((self.keys[self.current - 1].clone(), self.values[self.current - 1].clone()))
        }
    }
}

fn get_ini(path: &String) -> Ini {
    let conf_path = get_ini_path(&path);
    Ini::load_from_file(conf_path).expect("cannot open ini file.")
}

fn get_ini_path(path: &String) -> String {
    path.clone() + "\\conf.ini"
}

impl ConfIni {
    pub fn new(path: &String) -> ConfIni {
        let instance = ConfIni { ini_file: get_ini(path) };
        instance
    }

    pub fn get_ini_value(&self, key: String) -> Option<String> {
        for (sec, prop) in self.ini_file.iter() {
            if *sec == Some("kihon".to_string()) {
                for (k, v) in prop.iter() {
                    if *k == key {
                        if &*v != "" {
                            return Some(String::from_str(&*v.to_string()).expect("failed read ini file."));
                        }
                    }
                }
            }
        }

        None
    }

    pub fn set_ini_value(&mut self, exe_path: &String, mut write_param: IniCollection) -> Result<(), String> {
        let conf_path = get_ini_path(exe_path);
        let mut ini_file = self.ini_file.clone();

        loop {
            match write_param.next() {
                Some(kvp) => { ini_file.set_to(Some("kihon"), kvp.0, kvp.1); },
                None => { break; },
            }
        }

        match ini_file.write_to_file(&conf_path) {
            Ok(_) => {
                return Ok(());
            },
            Err(_) => {
                return Err("conf.ini write error ".to_string());
            }
        }
    }
}