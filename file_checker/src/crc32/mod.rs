pub struct Crc32 {
    table:[u32; 256],
    pub crc32: u32,
}

impl Crc32 {
    pub fn new() -> Crc32 {
        Crc32 {
            table: init_crc32(),
            crc32: 0xffffffff,
        }
    }

    pub fn update(&mut self, buffer: &Vec<u8>) -> u32 {
        let mut result = self.crc32;

        for i in 0..buffer.len() {
            result = (result >> 8) ^self.table[(buffer[i as usize] ^(result as u8 & 0xFF)) as usize];
        }

        self.crc32 = !result;
        !result
    }

}

fn init_crc32() -> [u32; 256]{
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