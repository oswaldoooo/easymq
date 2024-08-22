use std::{error::Error, io::Write};

pub enum Act {
    WriteSym = 1,
    Ack = 2,
}
pub trait Log {
    fn push(&mut self,id:u32, content: &str) -> Result<(), Box<dyn Error>>;
    fn read_latest(&mut self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn ack(&mut self,id:u32) -> Result<(), Box<dyn Error>>;
}
pub trait LogBuilder<T> where T:Log {
    fn build(&self, topic: &str) -> Result<T, Box<dyn Error>>;
}

pub struct FileLog {
    fd: std::fs::File,
}
impl Log for FileLog {
    fn push(&mut self,id:u32, content: &str) -> Result<(), Box<dyn Error>> {
        use std::io::Write;
        let lenbuff = (content.len() as u16).to_be_bytes();
        let mut vbuff: Vec<u8> = Vec::with_capacity(content.len() + 3);
        vbuff.push(Act::WriteSym as u8);
        vbuff.write(&lenbuff).unwrap();
        vbuff.write(content.as_bytes()).unwrap();
        self.fd.write_all(&vbuff)?;
        Ok(())
    }

    fn read_latest(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    fn ack(&mut self,id:u32) -> Result<(), Box<dyn Error>> {
        let mut vbuff = Vec::with_capacity(5);
        vbuff.write(&[Act::Ack as u8]);
        vbuff.write(&id.to_be_bytes());
        self.fd.write_all(&vbuff)?;
        Ok(())
    }
}
pub struct FileLogBuilder {
    pub root_path: String,
}
impl FileLogBuilder {
    pub fn new(rpath: &str) -> Self {
        Self {
            root_path: rpath.to_string(),
        }
    }
}
impl LogBuilder<FileLog> for FileLogBuilder {
    fn build(&self, topic: &str) -> Result<FileLog, Box<dyn Error>> {
        let rpath = std::path::Path::new(self.root_path.as_str()).join(topic);
        let fd = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(rpath)?;
        Ok(FileLog { fd: fd })
    }
}