
pub struct EndCentDirHeader  {  
    pub signature: u32,
    pub disknum: u16,
    pub startdisknum: u16,  
    pub diskdirentry: u16,
    pub direntry: u16,
    pub dirsize: u32,
    pub startpos: u32,
    pub commentlen: u16, 
    pub comment: String,
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
