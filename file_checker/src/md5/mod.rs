use std::mem;

const BLOCK_SIZE: usize = 16;
const BLOCK_SIZE_BYTES: usize = BLOCK_SIZE * 4;

pub struct Context {
	h: [u32; 4],
	leftover: [u8; BLOCK_SIZE_BYTES],
	leftoverlen: u32,
	hashedlen: i64,
}

impl Context {
	pub fn new() -> Context{
		Context {
			h: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
			leftover: [0x00; BLOCK_SIZE_BYTES],
			leftoverlen: 0,
			hashedlen: 0
		}
	}

	fn hash_block(&mut self, block: [u32; BLOCK_SIZE]) {
		let mut a = self.h[0];
		let mut b = self.h[1];
		let mut c = self.h[2];
		let mut d = self.h[3];

		// flip the endian-ness of the input block
		let x:[u32; BLOCK_SIZE] = self.decode(block);

		// Round 1
		a = ff(a, b, c, d, x[ 0],  7, 0xd76aa478); /* 1 */
		d = ff(d, a, b, c, x[ 1], 12, 0xe8c7b756); /* 2 */
		c = ff(c, d, a, b, x[ 2], 17, 0x242070db); /* 3 */
		b = ff(b, c, d, a, x[ 3], 22, 0xc1bdceee); /* 4 */
		a = ff(a, b, c, d, x[ 4],  7, 0xf57c0faf); /* 5 */
		d = ff(d, a, b, c, x[ 5], 12, 0x4787c62a); /* 6 */
		c = ff(c, d, a, b, x[ 6], 17, 0xa8304613); /* 7 */
		b = ff(b, c, d, a, x[ 7], 22, 0xfd469501); /* 8 */
		a = ff(a, b, c, d, x[ 8],  7, 0x698098d8); /* 9 */
		d = ff(d, a, b, c, x[ 9], 12, 0x8b44f7af); /* 10 */
		c = ff(c, d, a, b, x[10], 17, 0xffff5bb1); /* 11 */
		b = ff(b, c, d, a, x[11], 22, 0x895cd7be); /* 12 */
		a = ff(a, b, c, d, x[12],  7, 0x6b901122); /* 13 */
		d = ff(d, a, b, c, x[13], 12, 0xfd987193); /* 14 */
		c = ff(c, d, a, b, x[14], 17, 0xa679438e); /* 15 */
		b = ff(b, c, d, a, x[15], 22, 0x49b40821); /* 16 */

		// Round 2
		a = gg(a, b, c, d, x[ 1],  5, 0xf61e2562); /* 17 */
		d = gg(d, a, b, c, x[ 6],  9, 0xc040b340); /* 18 */
		c = gg(c, d, a, b, x[11], 14, 0x265e5a51); /* 19 */
		b = gg(b, c, d, a, x[ 0], 20, 0xe9b6c7aa); /* 20 */
		a = gg(a, b, c, d, x[ 5],  5, 0xd62f105d); /* 21 */
		d = gg(d, a, b, c, x[10],  9,  0x2441453); /* 22 */
		c = gg(c, d, a, b, x[15], 14, 0xd8a1e681); /* 23 */
		b = gg(b, c, d, a, x[ 4], 20, 0xe7d3fbc8); /* 24 */
		a = gg(a, b, c, d, x[ 9],  5, 0x21e1cde6); /* 25 */
		d = gg(d, a, b, c, x[14],  9, 0xc33707d6); /* 26 */
		c = gg(c, d, a, b, x[ 3], 14, 0xf4d50d87); /* 27 */
		b = gg(b, c, d, a, x[ 8], 20, 0x455a14ed); /* 28 */
		a = gg(a, b, c, d, x[13],  5, 0xa9e3e905); /* 29 */
		d = gg(d, a, b, c, x[ 2],  9, 0xfcefa3f8); /* 30 */
		c = gg(c, d, a, b, x[ 7], 14, 0x676f02d9); /* 31 */
		b = gg(b, c, d, a, x[12], 20, 0x8d2a4c8a); /* 32 */

		// Round 3
		a = hh(a, b, c, d, x[ 5],  4, 0xfffa3942); /* 33 */
		d = hh(d, a, b, c, x[ 8], 11, 0x8771f681); /* 34 */
		c = hh(c, d, a, b, x[11], 16, 0x6d9d6122); /* 35 */
		b = hh(b, c, d, a, x[14], 23, 0xfde5380c); /* 36 */
		a = hh(a, b, c, d, x[ 1],  4, 0xa4beea44); /* 37 */
		d = hh(d, a, b, c, x[ 4], 11, 0x4bdecfa9); /* 38 */
		c = hh(c, d, a, b, x[ 7], 16, 0xf6bb4b60); /* 39 */
		b = hh(b, c, d, a, x[10], 23, 0xbebfbc70); /* 40 */
		a = hh(a, b, c, d, x[13],  4, 0x289b7ec6); /* 41 */
		d = hh(d, a, b, c, x[ 0], 11, 0xeaa127fa); /* 42 */
		c = hh(c, d, a, b, x[ 3], 16, 0xd4ef3085); /* 43 */
		b = hh(b, c, d, a, x[ 6], 23,  0x4881d05); /* 44 */
		a = hh(a, b, c, d, x[ 9],  4, 0xd9d4d039); /* 45 */
		d = hh(d, a, b, c, x[12], 11, 0xe6db99e5); /* 46 */
		c = hh(c, d, a, b, x[15], 16, 0x1fa27cf8); /* 47 */
		b = hh(b, c, d, a, x[ 2], 23, 0xc4ac5665); /* 48 */

		// Round 4
		a = ii(a, b, c, d, x[ 0],  6, 0xf4292244); /* 49 */
		d = ii(d, a, b, c, x[ 7], 10, 0x432aff97); /* 50 */
		c = ii(c, d, a, b, x[14], 15, 0xab9423a7); /* 51 */
		b = ii(b, c, d, a, x[ 5], 21, 0xfc93a039); /* 52 */
		a = ii(a, b, c, d, x[12],  6, 0x655b59c3); /* 53 */
		d = ii(d, a, b, c, x[ 3], 10, 0x8f0ccc92); /* 54 */
		c = ii(c, d, a, b, x[10], 15, 0xffeff47d); /* 55 */
		b = ii(b, c, d, a, x[ 1], 21, 0x85845dd1); /* 56 */
		a = ii(a, b, c, d, x[ 8],  6, 0x6fa87e4f); /* 57 */
		d = ii(d, a, b, c, x[15], 10, 0xfe2ce6e0); /* 58 */
		c = ii(c, d, a, b, x[ 6], 15, 0xa3014314); /* 59 */
		b = ii(b, c, d, a, x[13], 21, 0x4e0811a1); /* 60 */
		a = ii(a, b, c, d, x[ 4],  6, 0xf7537e82); /* 61 */
		d = ii(d, a, b, c, x[11], 10, 0xbd3af235); /* 62 */
		c = ii(c, d, a, b, x[ 2], 15, 0x2ad7d2bb); /* 63 */
		b = ii(b, c, d, a, x[ 9], 21, 0xeb86d391); /* 64 */

		self.h[0] += a;
		self.h[1] += b;
		self.h[2] += c;
		self.h[3] += d;
	}

	fn copy_buffer(&mut self, dest_index: u32, buffer: &Vec<u8>, length: u32) {
		let mut i = dest_index as usize;
		for k in 0..length as usize {
			self.leftover[i] = buffer[k];
			i += 1;
		}
	}

	fn fill_block(&mut self, mut block: [u32; BLOCK_SIZE]) {
		let mut i = 0;
		for k in 0..BLOCK_SIZE {
			block[k] = bits_to_int(&self.leftover, i);
			i += 4;
		}
		self.hash_block(block);
	}

	pub fn update(&mut self, input: &mut Vec<u8>) {
		let len:u32 = input.len() as u32;
		let mut block: [u32; BLOCK_SIZE] = unsafe{ mem::uninitialized() };
		let mut input_index: u32 = 0;

		// if there is a leftover partial block, start with that
		if self.leftover.len() != 0 {
			// if we still can't fill a complete block
			if len + self.leftoverlen < BLOCK_SIZE_BYTES as u32 {
				// store the input and bail out
				let dest_index = self.leftoverlen;
				self.copy_buffer(dest_index, &input, len);
				self.leftoverlen += len;
				return;
			}

			// fill up the partial block
			input_index += self.leftover.len() as u32 - self.leftoverlen;
			let dest_index = self.leftoverlen; 
			self.copy_buffer(dest_index, input, input_index);

			// convert the block to integers
			self.fill_block(block);
			self.hashedlen += BLOCK_SIZE_BYTES as i64;
		}

		// if we have enough input for a block
		while len - input_index >= BLOCK_SIZE_BYTES as u32 {
			// convert the block to integers
			for k in 0..BLOCK_SIZE {
				block[k] = bits_to_int_vec(input, input_index as usize);
				input_index += 4;
			}
			self.hash_block(block);
			self.hashedlen += BLOCK_SIZE_BYTES as i64;
		}

		// store leftover partial block
		self.leftoverlen = len - input_index;
		if self.leftoverlen > 0 {
			self.leftover = unsafe { mem::uninitialized() };
			for u in 0..self.leftoverlen as u32 {
				self.leftover[u as usize] = input[input_index as usize];
				input_index += 1;
			}
		} else {
			self.leftover = unsafe { mem::uninitialized() };
		}
	}

	fn decode(&mut self, block: [u32; BLOCK_SIZE]) -> [u32; BLOCK_SIZE] {
		let mut x: [u32; BLOCK_SIZE] = unsafe { mem::uninitialized() };
		for i in 0..BLOCK_SIZE {
			x[i] = bits_rev(block[i as usize]);
		}
		x
	}

	pub fn do_final(&mut self) -> Vec<u8> {
		let block: [u32; BLOCK_SIZE] = unsafe{ mem::uninitialized() };
		self.hashedlen += self.leftoverlen as i64;

		// we might have one or two more blocks to hash at this point.
		// If the leftover partial block is smaller than 56 bytes, we
		// can append the padding bits and the length to it. Otherwise,
		// the padding and length will extend into another block.

		// if partial block exists
		if self.leftover.len() != 0 {
			// tag the end of input
			// leftoverLen should be < leftover.length
			self.leftover[self.leftoverlen as usize] = 0x80;
			self.leftoverlen += 1;

			if self.leftoverlen > BLOCK_SIZE_BYTES as u32 - 8 {
				//set 0x00 to limit
				while self.leftoverlen < BLOCK_SIZE_BYTES as u32 {
					self.leftover[self.leftoverlen as usize] = 0x00;
					self.leftoverlen += 1;
				}

				// convert the block to integers
				self.fill_block(block);
				self.leftover = unsafe { mem::uninitialized() };
				self.leftoverlen = 0;
			}
		} else {
			self.leftover = unsafe { mem::uninitialized() };
			self.leftover[0] = 0x80;
			self.leftoverlen = 1;
		}

		//set 0x00 to border
		while self.leftoverlen < (BLOCK_SIZE_BYTES as u32 - 8) {
			self.leftover[self.leftoverlen as usize] = 0x00;
			self.leftoverlen += 1;
		}
		
		//convert current hashed value
		let arr = self.leftover.to_vec();
		let ret_array = bits_to_bytes_big_endian_long((self.hashedlen * 8) as u64, &arr, self.leftoverlen as usize);
		for i in 0..ret_array.len() {
			self.leftover[i as usize] = ret_array[i as usize];
		}

		// convert the block to integers
		self.fill_block(block);

		// make a 16 byte array (128 bits)
		let byte_array: [u8; 16] = unsafe { mem::uninitialized() };
		let mut byte_vec = byte_array.to_vec();
		for k in 0..4 {
			byte_vec = bits_to_bytes_big_endian(self.h[k], &byte_vec, k*4);
		}

		byte_vec
	}
}


fn f(x:u32, y:u32, z:u32) -> u32 {
	(x & y) | ((!x) & z)
}

fn g(x:u32, y:u32, z:u32) -> u32 {
	(x & z) | (y & (!z))
}

fn h(x:u32, y:u32, z:u32) -> u32 {
	(x ^ y ^ z)
}

fn i(x:u32, y:u32, z:u32) -> u32 {
	(y ^ (x | (!z)))
}

fn ff(a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
	bits_left_rotate(a + f(b, c, d) + x + ac, s) + b
}

fn gg(a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
	bits_left_rotate(a + g(b, c, d) + x + ac, s) + b
}

fn hh(a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
	bits_left_rotate(a + h(b, c, d) + x + ac, s) + b
}

fn ii(a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
	bits_left_rotate(a + i(b, c, d) + x + ac, s) + b
}

fn bits_left_rotate(a:u32, s:u32) -> u32 {
	let w = s % 32;
	(a << w) | (a >> (32-w))
}


fn bits_rev(x: u32) -> u32 {
	(x >> 24) | ((x >> 8) &0x0000FF00) | ((x << 8) & 0x00FF0000) | (x << 24)
}


fn bits_to_int_vec(b: &mut Vec<u8>, offset: usize) -> u32 {
	let b0: u32 = b[offset] as u32;
	let b1: u32 = b[offset + 1] as u32;
	let b2: u32 = b[offset + 2] as u32;
	let b3: u32 = b[offset + 3] as u32;
	b0 << 24 | b1 << 16 | b2 << 8 | b3
	//(((b[offset] & 0xFF) << 24) | ((b[offset + 1] & 0xFF) << 16) | ((b[offset + 2] & 0xFF) << 8) | (b[offset + 3] & 0xFF)) as u32
}

fn bits_to_int(b: &[u8; BLOCK_SIZE_BYTES], offset: usize) -> u32 {
	let b0: u32 = b[offset] as u32;
	let b1: u32 = b[offset + 1] as u32;
	let b2: u32 = b[offset + 2] as u32;
	let b3: u32 = b[offset + 3] as u32;
	b0 << 24 | b1 << 16 | b2 << 8 | b3
	//(((b[offset] & 0xFF) << 24) | ((b[offset + 1] & 0xFF) << 16) | ((b[offset + 2] & 0xFF) << 8) | (b[offset + 3] & 0xFF)) as u32
}

fn bits_to_bytes_big_endian_long(a: u64, source: &Vec<u8>, offset: usize) -> Vec<u8> {
	let mut bytes = source.clone();
	let mut cnv_source = a;

	for i in 0..8 {
		let cnv: u64 = cnv_source & 0x000000FF;
		bytes[offset + i] = cnv as u8;
		cnv_source = cnv_source >> 8;
	}

	bytes
}

fn bits_to_bytes_big_endian(a: u32, source: &Vec<u8>, offset: usize) -> Vec<u8> {
	let mut bytes = source.clone();

	let cnv1: u32 = (a >> 24) & 0x000000FF;
	let cnv2: u32 = (a >> 16) & 0x000000FF;
	let cnv3: u32 = (a >> 8) & 0x000000FF;
	let cnv4: u32 = a & 0x000000FF;
	bytes[offset + 3] = cnv1 as u8; 
	bytes[offset + 2] = cnv2 as u8;
	bytes[offset + 1] = cnv3 as u8;
	bytes[offset] = cnv4 as u8;

	bytes
}
