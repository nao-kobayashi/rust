extern crate md5;

use std::env;
use std::fs;
use std::io::{BufReader, Read};

//check windows
//certutil -hashfile D:hoge.exe MD5
fn main() {
    const BUF_SIZE: usize = 1048576;
    let args: Vec<String> = env::args().collect();
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let mut md5_context = md5::Context::new();
    let mut calc_to = Vec::new();

    let mut reader = BufReader::with_capacity(BUF_SIZE, (fs::File::open(&args[1])).unwrap());
    loop { 
        match reader.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                calc_to.clear();
                let mut count = 0;
                while count < n {
                    calc_to.push(buffer[count]);
                    count += 1;
                }
                md5_context.consume(&calc_to);
            },
            Err(e) => {
                println!("read error {:?}", e);
                return;
            }
        }
    };

    let digest = md5_context.compute();
    println!("md5 is {:x}", digest);
}
