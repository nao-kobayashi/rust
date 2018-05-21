extern crate chrono;
extern crate deflate;
extern crate zipr;

use deflate::deflate_bytes;

use std::{env, process};
use std::fs::*;
use std::path::PathBuf;
use chrono::prelude::*;
use std::io::{BufReader, Read, BufWriter, Write};

use zipr::zip_header::ZipHeader;
use zipr::central_dir_header::CentralDirHeader;
use zipr::end_cent_dir_header::EndCentDirHeader;
use zipr::util::*;

const BUF_SIZE: usize = 1048576;


// 日付を取得  
fn get_dos_date(year: u16, month: u16, day: u16) -> u16 {
     (year - 1980 << 9) | month << 5 | day
}

// 時刻を取得  
fn get_dos_time(hour: u16, muinute: u16, second: u16) -> u16 {
    hour << 11 | muinute << 5 | second >> 1
}

fn main() {

    let args: Vec<String> = env::args().skip(1).collect();
    let mut write_source: Vec<ZipHeader> = Vec::new();
    let mut write_source_central: Vec<CentralDirHeader> = Vec::new();

    if args.len() < 2 {
        println!("パラメータの数が違います。");
        process::exit(0);
    }

    let zip_output_path = &args[0].clone();
    let len = *&args.len() as u32;

    for i in 1..len   {
        let path = PathBuf::from(&args[i as usize]);
        if path.is_file() {
            //ファイルヘッダ
            let mut header = ZipHeader::new();

            //ファイルの作成日付取得
            let meta = metadata(&path).unwrap();
            let crt_date: DateTime<Local> = meta.created().unwrap().into();

            //ヘッダに値をセット
            header.filetime = get_dos_time(crt_date.hour() as u16, crt_date.minute() as u16, crt_date.second() as u16);
            header.filedate = get_dos_date(crt_date.year() as u16, crt_date.month() as u16, crt_date.day() as u16);
            let file_name_info = get_filename(&path);
            header.fnamelen = file_name_info.0 as u16;
            header.filename = file_name_info.1;

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


            //圧縮処理
            let comp_file = deflate_bytes(file_bytes.as_slice());
            //圧縮元のファイルのCRCを計算する。
            header.get_crc32(&file_bytes, 0xffffffff);
            header.compsize = comp_file.len() as u32;
            header.filedata = comp_file;

            let mut central = CentralDirHeader::new();
            central.copy_to_centraldir(&header);
            write_source.push(header);
            write_source_central.push(central);
        }
    }

    //書込み
    let mut index = 0;
    let mut pos_archive:usize = 0;
    let mut writer = BufWriter::new(File::create(zip_output_path).unwrap());
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
