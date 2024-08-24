use std::{error::Error, sync::Arc};
use tokio::io::{AsyncWriteExt,AsyncWrite};
use tokio::sync::{Mutex, Notify};

pub enum Act {
    WriteSym = 1,
    Ack = 2,
}
pub trait Log {
    fn push(
        &mut self,
        id: u32,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn read_latest(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Box<dyn Error>>> + Send;
    fn ack(
        &mut self,
        id: u32,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn Error+'static>>> + Send;
}
pub trait LogBuilder<T>
where
    T: Log + Send + 'static,
{
    fn build(&self, topic: &str) -> impl std::future::Future<Output = Result<T, Box<dyn Error>>>+Send;
}

pub struct FileLog {
    //delay write(ms).if it's equal zero. it is live mode
    delay_duration: u64,
    fd: Arc<Mutex<tokio::fs::File>>,
    buff: Arc<Mutex<Vec<u8>>>,
    notifier: Arc<Notify>,
}
impl Log for FileLog {
    async fn push(&mut self, _id: u32, content: &str) -> Result<(), Box<dyn Error>> {
        // use std::io::Write;
        let lenbuff = (content.len() as u16).to_be_bytes();
        let mut vbuff: Vec<u8> = Vec::with_capacity(content.len() + 3);
        vbuff.push(Act::WriteSym as u8);
        vbuff.write(&lenbuff).await.unwrap();
        vbuff.write(content.as_bytes()).await.unwrap();
        if self.delay_duration == 0 {
            let mut fdll = self.fd.lock().await;
            fdll.write_all(&vbuff).await?;
        } else {
            self.buff.lock().await.write_all(&vbuff).await?;
        }
        Ok(())
    }

    async fn read_latest(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        todo!()
    }

    async fn ack(&mut self, id: u32) -> Result<(), Box<dyn Error>> {
        let mut vbuff = Vec::with_capacity(5);
        vbuff.write(&[Act::Ack as u8]).await?;
        vbuff.write(&id.to_be_bytes()).await?;
        if self.delay_duration == 0 {
            self.fd.lock().await.write_all(&vbuff).await?;
        } else {
            self.buff.lock().await.write_all(&vbuff).await?;
        }

        Ok(())
    }
}
pub struct FileLogBuilder {
    pub root_path: String,
    pub delay_duration: u64,
}
impl FileLogBuilder {
    pub fn new(rpath: &str, delay_duration: u64) -> Self {
        Self {
            root_path: rpath.to_string(),
            delay_duration: delay_duration,
        }
    }
}
impl LogBuilder<FileLog> for FileLogBuilder {
    async fn build(&self, topic: &str) -> Result<FileLog, Box<dyn Error>> {
        let rpath = std::path::Path::new(self.root_path.as_str()).join(topic);
        let fd = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(rpath).await?;
        let fl = FileLog {
            fd: Arc::new(Mutex::new(fd)),
            delay_duration: self.delay_duration,
            buff: Arc::new(Mutex::new(Vec::new())),
            notifier: Arc::new(Notify::new()),
        };
        if self.delay_duration > 0 {
            tokio::spawn(FileLog::delay_write(
                fl.buff.clone(),
                fl.notifier.clone(),
                fl.fd.clone(),
                self.delay_duration,
            ));
        }
        Ok(fl)
    }
}

impl FileLog {
    async fn delay_write(
        buff: Arc<Mutex<Vec<u8>>>,
        notify: Arc<Notify>,
        fd: Arc<Mutex<tokio::fs::File>>,
        delay: u64,
    ) {
        loop {
            notify.notified().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            let mut fdll = fd.lock().await;
            let buffll = buff.lock().await;
            if let Err(err) = fdll.write_all(buffll.as_slice()).await {
                eprintln!("write to file failed {err}");
            }
        }
    }
}
