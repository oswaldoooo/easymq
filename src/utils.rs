use std::{error::Error, fmt::Debug, io::Read};

use crate::log;

pub struct FileLogChecker {}
impl FileLogChecker {
    pub fn new() -> Self {
        Self {}
    }
    pub fn check(&self, rpath: &str) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut fd = std::fs::OpenOptions::new().read(true).open(rpath)?;
        let mut buff = [0u8; 1 << 10];
        let mut lenbuff = [0u8; 2];
        let mut result = Vec::new();
        let mut len = 0;
        loop {
            if let Err(err) = fd.read_exact(&mut buff[0..1]) {
                break;
            }
            let act = buff[0];
            match act {
                1 => {
                    //write act
                    fd.read_exact(&mut lenbuff)?;
                    len = u16::from_be_bytes(lenbuff);
                    fd.read_exact(&mut buff[0..len as usize])?;
                    let vec = buff[0..len as usize].to_vec();
                    result.push(Value::Message(String::from_utf8(vec)?));
                }
                2 => {
                    //ack
                    result.push(Value::Ack);
                }
                _ => panic!("not impelement act code {act}"),
            }
        }
        Ok(result)
    }
}
pub enum Value {
    Ack,
    Message(String),
}
impl Debug for Value{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ack => write!(f, "Ack"),
            Self::Message(arg0) => f.debug_tuple("Message").field(arg0).finish(),
        }
    }
}