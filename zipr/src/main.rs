extern crate chrono;
extern crate flate2;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;


use std::env;
use std::fs::*;
use std::path::PathBuf;
use std::mem::transmute;
use chrono::prelude::*;
use std::io::{BufReader, Read, BufWriter, Write};

struct ZipHeader {
    signature: u32,
    needver: u16,
    option: u16,
    comptype: u16,
    filetime: u16,
    filedate: u16,
    crc32: u32,
    compsize: u32,
    uncompsize: u32,
    fnamelen: u16,
    extralen: u16,
    crc_table: [u32; 256],
    filename: String,
    extradata: u8,
    filedata: Vec<u8>
}

struct EndCentDirHeader  {  
    signature: u32,
    disknum: u16,
    startdisknum: u16,  
    diskdirentry: u16,
    direntry: u16,
    dirsize: u32,
    startpos: u32,
    commentlen: u16, 
    comment: String,
}



struct CentralDirHeader  
{  
    signature: u32,
    madever: u16,
    needver: u16,
    option: u16,
    comptype: u16,
    filetime: u16,
    filedate: u16,
    crc32: u32,
    compsize: u32,
    uncompsize: u32,
    fnamelen: u16,
    extralen: u16,
    commentlen: u16,
    disknum: u16,
    inattr: u16,
    outattr: u32,
    headerpos: u32,
    filename: String,
    extradata: u8,
    comment: String,
}

impl ZipHeader {
    pub fn new() -> ZipHeader {
        let def_crc32 = init_crc32();

        ZipHeader {
            signature: 0x04034B50,
            needver: 20,
            option: 0,
            comptype: 8,
            filetime: 0,
            filedate: 0,
            crc32: 0,
            compsize: 0,
            uncompsize: 0,
            fnamelen: 0,
            extralen: 0,
            crc_table: def_crc32,
            filename: "".to_string(),
            extradata: 0,
            filedata: Vec::new()
        }
    }

}


impl EndCentDirHeader  {  
    pub fn new () -> EndCentDirHeader {
        EndCentDirHeader{
            signature: 0x06054B50,
            disknum: 0,
            startdisknum: 0,  
            diskdirentry: 0,
            direntry: 0,
            dirsize: 0,
            startpos: 0,
            commentlen: 0,
            comment: "".to_string()

        }
    }
}


impl CentralDirHeader {
    fn new() -> CentralDirHeader{
        CentralDirHeader {
            signature: 0x02014B50,
            madever: 20,
            needver: 20,
            option: 0,
            comptype: 8,
            filetime: 0,
            filedate: 0,
            crc32: 0,
            compsize: 0,
            uncompsize: 0,
            fnamelen: 0,
            extralen: 0,
            commentlen: 0,
            disknum: 0,
            inattr: 0,
            outattr: 0,
            headerpos: 0,
            filename: "".to_string(),
            extradata: 0,
            comment: "".to_string(),
        }
    }
}


fn init_crc32() -> [u32; 256] {
    let poly: u32 = 0xEDB88320;
    let mut crc_table: [u32; 256] = [0; 256];

    for i in 0..256 {
        let mut u = i;

        for _j in 0..8 {
            if u & 0x1 == 1 {
                u = (u >> 1) ^ poly;
            } else {
                u >>= 1;
            }
        }

        crc_table[i as usize] = u;
    }
    
    crc_table
}

fn get_crc32(buffer: &Vec<u8>, crc32_start: u32, table: [u32; 256]) -> u32 {
    let mut result = crc32_start;

    for i in 0..buffer.len() {
        result = (result >> 8) ^table[(buffer[i as usize] ^(result as u8 & 0xFF)) as usize];
    }

    !result
}

// 日付を取得  
fn get_dos_date(year: u16, month: u16, day: u16) -> u16 {
     (year - 1980 << 9) | month << 5 | day
}

// 時刻を取得  
fn get_dos_time(hour: u16, muinute: u16, second: u16) -> u16 {
    hour << 11 | muinute << 5 | second >> 1
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
const BUF_SIZE: usize = 1048576;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut write_source: Vec<ZipHeader> = Vec::new();
    let mut write_source_central: Vec<CentralDirHeader> = Vec::new();

    for file in args {
        let path = PathBuf::from(&file);
        if path.is_file() {
            //ファイルヘッダ
            let mut header = ZipHeader::new();

            //ファイルの作成日付取得
            let meta = metadata(&path).unwrap();
            let crt_date: DateTime<Local> = meta.created().unwrap().into();

            //ヘッダに値をセット
            header.filetime = get_dos_time(crt_date.hour() as u16, crt_date.minute() as u16, crt_date.second() as u16);
            header.filedate = get_dos_date(crt_date.year() as u16, crt_date.month() as u16, crt_date.day() as u16);
            header.fnamelen = path.file_name().unwrap().to_os_string().len() as u16;
            header.filename = path.file_name().unwrap().to_os_string().into_string().unwrap();

            //圧縮したら変更する。
            header.uncompsize = meta.len() as u32;

            //ファイル読み込み
            let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];
            let mut file_bytes: Vec<u8> = Vec::new();
            let mut reader = BufReader::with_capacity(BUF_SIZE, (File::open(path)).unwrap());
            loop { 
                match reader.read(&mut buffer) {
                    Ok(n) => {
                        if n == 0 { break; }
                        file_bytes.append(&mut buffer[0..n].to_vec());
                    },
                    Err(e) => {
                        println!("read error {:?}", e);
                        return;
                    }
                }
            };



{
            //圧縮処理 正しく動かない。
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(file_bytes.as_slice()).unwrap();
            let comp_file = encoder.finish().unwrap();
            //let comp_file = file_bytes;


            header.crc32 = get_crc32(&comp_file, 0xffffffff, header.crc_table);
            header.compsize = comp_file.len() as u32;
            header.filedata = comp_file;
}

            let mut central = CentralDirHeader::new();
            copy_to_centraldir(&mut central, &header);
            write_source.push(header);
            write_source_central.push(central);
        }
    }

    //書込み
    let mut index = 0;
    let mut pos_archive:usize = 0;
    let mut writer = BufWriter::new(File::create("test.zip").unwrap());
    for mut file in write_source {
        write_source_central[index].headerpos = pos_archive as u32;
        index += 1;

        pos_archive += write_u32(&mut writer, file.signature);
        pos_archive += write_u16(&mut writer, file.needver);
        pos_archive += write_u16(&mut writer, file.option);
        pos_archive += write_u16(&mut writer, file.comptype);
        pos_archive += write_u16(&mut writer, file.filetime);
        pos_archive += write_u16(&mut writer, file.filedate);
        pos_archive += write_u32(&mut writer, file.crc32);
        pos_archive += write_u32(&mut writer, file.compsize);
        pos_archive += write_u32(&mut writer, file.uncompsize);
        pos_archive += write_u16(&mut writer, file.fnamelen);
        pos_archive += write_u16(&mut writer, file.extralen);
        pos_archive += write_u8(&mut writer, Vec::from(file.filename));
        //writer.write(file.extradate);
        pos_archive += write_u8(&mut writer, file.filedata);

    }

    //中間ディレクトリの書き込み
    let centran_dir_pos = pos_archive;
    let mut central_dir_len = 0;
    for file in write_source_central {
        central_dir_len += write_u32(&mut writer, file.signature);
        central_dir_len += write_u16(&mut writer, file.madever);
        central_dir_len += write_u16(&mut writer, file.needver);
        central_dir_len += write_u16(&mut writer, file.option);
        central_dir_len += write_u16(&mut writer, file.comptype);
        central_dir_len += write_u16(&mut writer, file.filetime);
        central_dir_len += write_u16(&mut writer, file.filedate);
        central_dir_len += write_u32(&mut writer, file.crc32);
        central_dir_len += write_u32(&mut writer, file.compsize);
        central_dir_len += write_u32(&mut writer, file.uncompsize);
        central_dir_len += write_u16(&mut writer, file.fnamelen);
        central_dir_len += write_u16(&mut writer, file.extralen);
        central_dir_len += write_u16(&mut writer, file.commentlen);
        central_dir_len += write_u16(&mut writer, file.disknum);
        central_dir_len += write_u16(&mut writer, file.inattr);
        central_dir_len += write_u32(&mut writer, file.outattr); 
        central_dir_len += write_u32(&mut writer, file.headerpos); 
        central_dir_len += write_u8(&mut writer, Vec::from(file.filename));
    }

    // 終端ヘッダの書き込み  
    let mut endheader = EndCentDirHeader::new();
    endheader.direntry = index as u16;
    endheader.diskdirentry = endheader.direntry;  
    endheader.startpos = centran_dir_pos as u32;
    endheader.dirsize = central_dir_len as u32;

    write_u32(&mut writer, endheader.signature);
    write_u16(&mut writer, endheader.disknum);
    write_u16(&mut writer, endheader.startdisknum);
    write_u16(&mut writer, endheader.diskdirentry);
    write_u16(&mut writer, endheader.direntry);
    write_u32(&mut writer, endheader.dirsize);
    write_u32(&mut writer, endheader.startpos);
    write_u16(&mut writer, endheader.commentlen);

    //flush zip file
    match writer.flush() {
        Ok(_) => {},
        Err(e) => panic!("file save error {:?}", e)
    };
}


fn copy_to_centraldir(dirheader: &mut CentralDirHeader, zipheader: &ZipHeader) {
	dirheader.needver = zipheader.needver;
	dirheader.option = zipheader.option;
	dirheader.comptype = zipheader.comptype;
	dirheader.filetime = zipheader.filetime;
	dirheader.filedate = zipheader.filedate;
	dirheader.crc32 = zipheader.crc32;
	dirheader.compsize = zipheader.compsize;
	dirheader.uncompsize = zipheader.uncompsize;
	dirheader.fnamelen = zipheader.fnamelen;
	dirheader.extralen = zipheader.extralen;

	dirheader.filename = zipheader.filename.clone();
	dirheader.extradata = zipheader.extradata;
}

fn write_u8(writer: &mut Write, value: Vec<u8>) -> usize {
    let buffer = value.as_slice();
    match writer.write(&buffer) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}

fn write_u16(writer: &mut Write, value: u16) -> usize {
    match writer.write(&cnv_u16_to_bytes(value)) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}

fn write_u32(writer: &mut Write, value: u32) -> usize {
    match writer.write(&cnv_u32_to_bytes(value)) {
        Ok(n) => n,
        Err(e) => panic!("fail write {:?}", e)
    }
}
