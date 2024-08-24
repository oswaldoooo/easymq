use super::Ack;
use crate::{log, Acker};
use std::net::SocketAddr;
use std::{error::Error, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};
static publish: u8 = 1;
static read_latest: u8 = 2;
static _ok: u8 = 1;
static _internal_error: u8 = 2;
pub async fn run<T, B>(
    svc: Arc<super::MessageQueueManager<T, B>>,
    bind: String,
)
where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    let listener = tokio::net::TcpListener::bind(bind).await.expect("tcp bind failed");
    while let Ok((conn, addr)) = listener.accept().await {
        tokio::spawn(handle::<T, B>(svc.clone(), conn, addr));
    }
}
async fn handle<T, B>(
    svc: Arc<super::MessageQueueManager<T, B>>,
    mut conn: TcpStream,
    addr: SocketAddr,
) where
    T: log::Log + Send + 'static,
    B: log::LogBuilder<T> + Send + 'static,
{
    let mut lenbuff = [0u8; 2];
    let mut buff = [0u8; 1500];
    loop {
        let result = conn.read_exact(&mut lenbuff[0..1]).await;
        if let Err(err) = result {
            // let _ = conn.shutdown().await;
            eprintln!("read connection error {err}");
            // conn.shutdown().await;
            return;
        }
        if lenbuff[0] == publish {
            let result = read_msg(&mut conn, &mut buff).await;
            if let Err(err) = result {
                eprintln!("read connection {err}");
                // let _=conn.shutdown().await;
                return;
            }
            let size = result.unwrap();
            let result = String::from_utf8(buff[0..size as usize].to_vec());
            if let Err(err) = result {
                eprintln!("topic is not convert string");
                // conn.shutdown().await;
                return;
            }
            let topic = result.unwrap();
            let result = read_msg(&mut conn, &mut buff).await;
            if let Err(err) = result {
                eprintln!("read connection {err}");
                // conn.shutdown().await;
                return;
            }
            let size = result.unwrap();
            let result = String::from_utf8(buff[0..size as usize].to_vec());
            if let Err(err) = result {
                eprintln!("topic is not convert string {err}");
                // conn.shutdown().await;
                return;
            }
            let content = result.unwrap();
            let _ = svc.push(topic, content).await;
            if let Err(err) = write_msg(&mut conn, _ok, "ok").await {
                eprintln!("connection failed {err}");
                // conn.shutdown().await;
                return;
            }
        } else if lenbuff[0] == read_latest {
            let result = read_msg(&mut conn, &mut buff).await;
            if let Err(err) = result {
                eprintln!("read connection {err}");
                // conn.shutdown().await;
                return;
            }
            let size = result.unwrap();
            let result = String::from_utf8(buff[0..size as usize].to_vec());
            if let Err(err) = result {
                eprintln!("topic is not convert string");
                // conn.shutdown().await;
                return;
            }
            let topic = result.unwrap();
            let result = svc.read_latest(topic).await;
            if let Err(err) = result {
                eprintln!("server errror {err}");
                // conn.shutdown().await;
                return;
            }
            let result = result.unwrap();
            // let mut result=Arc::new(Mutex::new(result));
            // let r2:Box<dyn Ack<T>>=Box::new(result);
            // let mut resultll=result.lock().await;
            // result.ack().await;
            let ans2 = write_and_ack(&mut conn, _ok, result).await;
            // ans2.is_err();
            if let Err(err) = ans2 {
                eprintln!("send msg to client is error {err}");
                continue;
            }
            // result.ack().await;
            // if let Ok(()) = ans {
            // let h=resultll.ack().await;

            // result.ack().await;
            // if let Err(err) = result.ack().await {
            //     eprintln!("ack failed {err}");
            // }
            // }
        } else {
            eprintln!("protocol error unknown act code {}", lenbuff[0]);
            // let _ = conn.shutdown().await;
            return;
        }
    }
}
async fn read_msg(conn: &mut TcpStream, buff: &mut [u8]) -> Result<u16, Box<dyn Error>> {
    let mut lenbuff = [0u8; 2];
    conn.read_exact(&mut lenbuff).await?;
    let size = u16::from_be_bytes(lenbuff);
    conn.read_exact(&mut buff[0..size as usize]).await?;
    Ok(size)
}
async fn write_msg(conn: &mut TcpStream, status: u8, msg: &str) -> Result<(), Box<dyn Error>> {
    let mut buff = Vec::with_capacity(msg.len() + 3);
    buff.write_all(&[status]).await.unwrap();
    let size = msg.len() as u16;
    buff.write_all(&size.to_be_bytes()).await.unwrap();
    buff.write_all(msg.as_bytes()).await.unwrap();
    let _ = conn.write_all(buff.as_slice()).await?;
    Ok(())
}
async fn write_and_ack<T>(
    conn: &mut TcpStream,
    status: u8,
    mut acker: Acker<T>,
) -> Result<(), Box<dyn Error>>
where
    T: log::Log + Send,
{
    let msg = acker.data.as_str();
    write_msg(conn, status, msg).await?;
    acker.ack().await?;
    Ok(())
}
//easymq client
pub struct Client {
    con: TcpStream,
}
impl Client {
    pub async fn connect(address: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            con: tokio::net::TcpStream::connect(address).await?,
        })
    }
    pub async fn publish(&mut self, topic: &str, content: &str) -> Result<(), Box<dyn Error>> {
        let mut buff = Vec::with_capacity(topic.len() + content.len() + 5);
        buff.push(publish);
        buff.write_all(&(topic.len() as u16).to_be_bytes())
            .await
            .unwrap();
        buff.write_all(topic.as_bytes()).await.unwrap();
        buff.write_all(&(content.len() as u16).to_be_bytes())
            .await
            .unwrap();
        buff.write_all(content.as_bytes()).await.unwrap();
        Ok(self.con.write_all(buff.as_slice()).await?)
    }
    pub async fn read_latest(&mut self, topic: &str) -> Result<String, Box<dyn Error>> {
        let mut buff = Vec::with_capacity(topic.len() + 3);
        buff.push(read_latest);
        buff.write_all(&(topic.len() as u16).to_be_bytes())
            .await
            .unwrap();
        buff.write_all(topic.as_bytes()).await.unwrap();
        self.con.write_all(buff.as_slice()).await?;
        let mut sigbuff = [0u8; 1];
        self.con.read_exact(&mut sigbuff).await?;
        if sigbuff[0] == _ok {
            let mut buff = [0u8; 1500];
            let size = read_msg(&mut self.con, &mut buff).await?;
            let v = buff[0..size as usize].to_vec();
            return Ok(String::from_utf8(v)?);
        } else if sigbuff[0] == _internal_error {
            //return internal error
        }
        panic!("unknown code {}", sigbuff[0])
    }
}
