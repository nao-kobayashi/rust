/// This code is a Copy from
/// https://employment.en-japan.com/engineerhub/entry/2017/07/19/110000#Rust%E3%81%A7%E5%AE%9F%E8%B7%B5%E7%9A%84%E3%81%AA%E5%AE%9F%E8%A3%85-%E3%81%9D%E3%81%AE1-rsgrep


extern crate regex;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::env;
use regex::Regex;

fn usage() {
    println!("rsgrep PATTERN FILENAME")
}

fn main() {
    
    //println!("{:?}", env::args().nth(0));
    //println!("{:?}", env::args().nth(1));
    //println!("{:?}", env::args().nth(2));

    let pattern = match env::args().nth(1) {
        Some(pattern) => pattern,
        None => {
            usage();
            return;
        }
    };

    let reg = match Regex::new(&pattern) {
        Ok(reg) => reg,
        Err(e) => {
            println!("invalid regexp {}: {}", pattern, e);
            return;
        }
    };

    let filename = match env::args().nth(2) {
        Some(filename) => filename,
        None => {
            usage();
            return;
        }
    };

    let file = match File::open(&filename) {
        Ok(file) => file,
        Err(e) => {
            println!("An error occurred while opening file {}:{}", filename, e);
            return;
        }
    };

    let mut rownum = 1;
    let input = BufReader::new(file);
    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                println!("An error occurred while reading line {}:{}", filename, e);
                return;
            }
        };

        if reg.is_match(&line) {
            println!("{}  :at line {}", line, rownum);
        }
        rownum+=1;
    }

}
