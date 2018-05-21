use ::zip_header::ZipHeader;

pub struct CentralDirHeader  
{  
    pub signature: u32,
    pub madever: u16,
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
    pub commentlen: u16,
    pub disknum: u16,
    pub inattr: u16,
    pub outattr: u32,
    pub headerpos: u32,
    pub filename: String,
    pub extradata: u8,
    pub comment: String,
}


impl CentralDirHeader {
    pub fn new() -> CentralDirHeader{
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

    pub fn copy_to_centraldir(&mut self, zipheader: &ZipHeader) {
        self.needver = zipheader.needver;
        self.option = zipheader.option;
        self.comptype = zipheader.comptype;
        self.filetime = zipheader.filetime;
        self.filedate = zipheader.filedate;
        self.crc32 = zipheader.crc32;
        self.compsize = zipheader.compsize;
        self.uncompsize = zipheader.uncompsize;
        self.fnamelen = zipheader.fnamelen;
        self.extralen = zipheader.extralen;

        self.filename = zipheader.filename.clone();
        self.extradata = zipheader.extradata;
    }

}