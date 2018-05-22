extern crate file_checker;

use std::env;
use std::fs;
use std::result::Result::{ Ok, Err };
use std::path::PathBuf;
use std::process;
use std::io::{BufReader, Read};
use file_checker::md5::Context;
use file_checker::crc32::Crc32;
use std::prelude::v1::Option::{ None, Some };

/*
releaseビルドで動かさないと
整数の桁チェックでオーバーフローする。
*/


const C_MD5: &'static str = "MD5";
const C_CRC32: &'static str = "CRC32";

struct Checker {
	md5_context: Context,
	crc32_context: Crc32,
	check_type: String,
	md5_result: Vec<u8>,
	crc32_result: u32
}

impl Checker {

	pub fn new() -> Checker {

		Checker{
			md5_context: Context::new(),
			crc32_context: Crc32::new(),
			check_type: "".to_string(),
			md5_result: vec![],
			crc32_result: 0
		}
	}

	pub fn set_check_type(&mut self,ct: String) -> Result<i32, String> {
		if ct.to_uppercase() == C_MD5
			|| ct.to_uppercase() == C_CRC32 {
			self.check_type = ct.to_uppercase();
			return Ok(0);
		} 

		Err("使用できるのは MD5 or CRC32 です。".to_string())
	}

	pub fn update(&mut self, input: &mut Vec<u8>) {
		if self.check_type == C_MD5 {
			self.md5_context.update(input);
		} else if self.check_type == C_CRC32 {
			self.crc32_result = self.crc32_context.update(input);
		}
	}

	fn get_result_vec(&mut self) -> Option<Vec<u8>> {
		if self.check_type == C_MD5 {
			self.md5_result = self.md5_context.do_final();
			return Some(self.md5_result.clone());
		}
		
		None
	}

	fn get_result_u32(&mut self) -> Option<u32> {
		if self.check_type == C_CRC32 {
			return Some(self.crc32_result);
		}

		None
	}

	pub fn print_stdout(&mut self) {
		if self.check_type == C_MD5 {
			match self.get_result_vec() {
				Some(v) => println!("{:x}", ByteBuf(&v)),
				None => println!("Parameter may be wrong."),
			};
		} else if self.check_type == C_CRC32 {
			match self.get_result_u32() {
				Some(c) => println!("{:x}", c),
				None => println!("Parameter may be wrong."),
			};
		}
	}
}

struct ByteBuf<'a>(&'a Vec<u8>);
impl<'a> std::fmt::LowerHex for ByteBuf<'a> {
	fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		for byte in self.0 {
			try!( fmtr.write_fmt(format_args!("{:02x}", byte)));
		}
		Ok(())
	}
}

fn main() {
	const BUF_SIZE: usize = 1048576;
	let args: Vec<String> = env::args().collect();
	let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

	if args.len() != 3 {
		println!("パラメータの数が違います。");
		process::exit(0);
	}

	let file = PathBuf::from(args[1].clone());
	let check_type = args[2].clone();

	if !file.is_file() {
		println!("第１引数に指定されたファイルが存在しません。");
		process::exit(0);
	}

	let mut checker = Checker::new();
	match checker.set_check_type(check_type) {
		Ok(_) => {},
		Err(s) => {
			println!("{:?}", s);
			process::exit(0);
		}
	}

	let mut reader = BufReader::with_capacity(BUF_SIZE, (fs::File::open(file)).unwrap());
	loop { 
		match reader.read(&mut buffer) {
			Ok(n) => {
				if n == 0 { break ; }

				let mut vec = buffer[0..n].to_vec();
				checker.update(&mut vec);
			},
			Err(e) => {
				println!("read error {:?}", e);
				return;
			}
		}
	};

	checker.print_stdout();
}


#[test]
fn compute() {
	let inputs = [
		"",
		"a",
		"abc",
		"message digest",
		"abcdefghijklmnopqrstuvwxyz",
		"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
		"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
	];

	let outputs = [
		"d41d8cd98f00b204e9800998ecf8427e",
		"0cc175b9c0f1b6a831c399e269772661",
		"900150983cd24fb0d6963f7d28e17f72",
		"f96b697d7cb7938d525a2f31aaf161d0",
		"c3fcd3d76192e4007dfb496cca67e13b",
		"d174ab98d277d9f5a5611c2c9f419d9f",
		"57edf4a22be3c955ac49da2e2107b67a",
	];

	for (input, &output) in inputs.iter().zip(outputs.iter()) {
		let mut context = Context::new();
		let vec = &mut input.as_bytes().to_vec();
		context.update(vec);
		println!("input string = {:?}", input);
		assert_eq!(format!("{:x}", ByteBuf(&context.do_final())), output);
	}
}
