
pub struct ZipHeader {
    pub signature: u32,
    pub needver: u16,
    pub option: u16,
    pub comptype: u16,
    pub filetime: u16,
    pub filedate: u16,
    pub crc32: u32,
    pub compsize: u32,
    pub uncompsize: u32,
    pub fnamelen: u16,
    pub extralen: u16,
    crc_table: [u32; 256],
    pub filename: String,
    pub extradata: u8,
    pub filedata: Vec<u8>
}



impl ZipHeader {
    pub fn new() -> ZipHeader {
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
            crc_table: init_crc32(),
            filename: "".to_string(),
            extradata: 0,
            filedata: Vec::new()
        }
    }

    pub fn get_crc32(&mut self, buffer: &Vec<u8>, crc32_start: u32) {
        let mut result = crc32_start;

        for i in 0..buffer.len() {
            result = (result >> 8) ^self.crc_table[(buffer[i as usize] ^(result as u8 & 0xFF)) as usize];
        }

        self.crc32 = !result;
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
