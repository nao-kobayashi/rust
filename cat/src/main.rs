use std::fs::File;
use std::io::{ stdout, Result, Read, Write };
use std::env::args;

fn main() {
/*
    println!("{}", args().count());
    println!("{}", args().nth(0).unwrap());
    println!("{}", args().nth(1).unwrap());
*/
    let paths: Vec<String> = args().skip(1).collect();
    if paths.is_empty() {
        panic!("file not given");
    }

    for path in paths{
        println!("file name:{}", path);

        let res = do_cat(path);
        if res.is_err() {
            println!("{}", res.unwrap_err());
        }
    }
}

const BUFFER_SIZE: usize = 2048;

fn do_cat(path: String) -> Result<()> {
    let stdout = stdout();
    let mut handle = stdout.lock();
    let mut in_buf = [0; BUFFER_SIZE];
    let mut reader = try!(File::open(&std::path::Path::new(&path)));
    
    loop {
        let n = match reader.read(&mut in_buf[..]) {
            Ok(n) if n == 0 => return Ok(()),
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        try!(handle.write(&in_buf[0..n]));
    }
}
