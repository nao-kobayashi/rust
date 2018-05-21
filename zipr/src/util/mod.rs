use std::io::Write;
use std::path::PathBuf;
use std::mem::transmute;

pub fn get_filename(path: &PathBuf) -> (usize, String) {
    let file_name = match path.file_name() {
        Some(s) => s,
        None => panic!("get file name fail.")
    };

    let os_string = match file_name.to_os_string().into_string() {
        Ok(s) => s,
        Err(e) => panic!("get file name fail. {:?}", e)
    };

    (os_string.len(), os_string)
}

pub fn write_u8(writer: &mut Write, value: Vec<u8>) -> usize {
    let buffer = value.as_slice();
    match writer.write(&buffer) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}

pub fn write_u16(writer: &mut Write, value: u16) -> usize {
    match writer.write(&cnv_u16_to_bytes(value)) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}

pub fn write_u32(writer: &mut Write, value: u32) -> usize {
    match writer.write(&cnv_u32_to_bytes(value)) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}

fn cnv_u32_to_bytes(val: u32) -> [u8; 4]{
    unsafe{ transmute(val) }
}

fn cnv_u16_to_bytes(val: u16) -> [u8; 2]{
    unsafe{ transmute(val) }
}
/*
fn cnv_u16_to_bytes(val: u16) -> [u8; 2]{
    unsafe{ transmute(val) }
}
*/