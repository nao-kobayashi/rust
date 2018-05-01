extern crate md5;

use std::env;
use std::fs;
use std::io::{BufReader, Read};

fn main() {
    const BUF_SIZE: usize = 1048576;
    let args: Vec<String> = env::args().collect();
    let mut result = Vec::new();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    let mut reader = BufReader::with_capacity(BUF_SIZE, (fs::File::open(&args[1])).unwrap());
    loop { 
        match reader.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                let mut count = 0;
                while count < n {
                    result.push(buffer[count]);
                    count += 1;
                }
            },
            Err(e) => {
                println!("read error {:?}", e);
                return;
            }
        }
    };

    let digest = md5::compute(result);
    println!("md5 is {:x}", digest);
}