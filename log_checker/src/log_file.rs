use std::fs::metadata;
use std::fs::File;
use std::io::{BufReader, BufRead};
use regex::Regex;
use std::error::Error;
use chrono::prelude::*;
use std::path::PathBuf;

pub struct LogFile {
    file_path: String,
    last_modify_date: DateTime<Local>,
    read_line: u32,
    regex_str: String,
}


impl LogFile {
    pub fn new(path: String, modify_date: DateTime<Local>, line: u32, regex: String) -> LogFile {
        LogFile {
            file_path: path,
            last_modify_date: modify_date,
            read_line: line,
            regex_str: regex,
        }
    }

   
    pub fn get_log_mod_date(&self) -> String {
        self.last_modify_date.to_string()
    }

    pub fn find_error(&mut self) -> Result<u32, String> {
        let s = "(?i)".to_string() + &self.regex_str;
        let reg = match Regex::new(&s) {
            Ok(reg) => reg,
            Err(_) => {
                return Err("invalid regexp.".to_string());
            }
        };

        let log: PathBuf = PathBuf::from(&self.file_path);
        if log.is_file() {
            let metadata = metadata(&log) ;
            let mod_date: DateTime<Local> = match metadata {
                Ok(metadata) => {
                    metadata.modified().unwrap().into()
                },
                Err(e) => panic!("file property not read. {:?}", e)
            };

            if mod_date != self.last_modify_date {
                self.read_line = 0;    
            }
            self.last_modify_date = mod_date;
        }

        let file = match File::open(&self.file_path) {
            Ok(file) => file,
            Err(_) => {
                return Err("An error occurred while opening file.".to_string());
            }
        };

        let mut rownum = 1;
        let input = BufReader::new(file);
        for line in input.lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    return Err("An error occurred while reading file.".to_string() + &e.description().to_string());
                }
            };

            if self.read_line < rownum {
                //println!("{:?}", &line);
                if reg.is_match(&line) {
                    //エラーが見つかった行を表示
                    println!("{} at line {}.", line.clone(), rownum);
                }
            }
            rownum += 1;
        }

        Ok(rownum)
    }
}