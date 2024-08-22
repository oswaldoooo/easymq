use std::{
    collections::{HashMap, LinkedList},
    error::Error,
    sync::Arc,
};
use tokio::sync::{Mutex, Notify};
pub mod log;
pub mod rest;
pub mod utils;
pub struct MessageQueue<T>
where
    T: log::Log+Send+'static,
{
    data: Arc<Mutex<LinkedList<Message>>>,
    log: Arc<Mutex<T>>,
    notifier: Notify,
}
impl<T> MessageQueue<T>
where
    T: log::Log+Send+'static,
{
    pub fn new(_log: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(LinkedList::new())),
            log: Arc::new(Mutex::new(_log)),
            notifier: Notify::new(),
        }
    }
    pub async fn push(&self, msg: Message) -> Result<(), Box<dyn Error>> {
        let mut logll = self.log.lock().await;
       //println!("log locked");
        let mut datall = self.data.lock().await;
       //println!("data locked");
        let id: u32 = if let Some(data) = datall.back() {
            data.id + 1
        } else {
            1
        };
        let _ = logll.push(id, msg.content.as_str()).await?;
        datall.push_front(msg);
       //println!("push front");
        self.notifier.notify_one();
       //println!("notifyed");
        Ok(())
    }
    pub async fn read_latest(&self, topic: &str) -> Result<Acker<T>, Box<dyn Error>> {
        let mut datall = self.data.lock().await;
       //println!("wait pop back");
        let mut ans = datall.pop_back();
       //println!("pop backed");
        while let None = ans {
            drop(datall);
           //println!("wait notify");
            self.notifier.notified().await;
           //println!("notify");
            datall = self.data.lock().await;
            ans = datall.pop_back();
        }
        let ans = ans.unwrap();
        // let mut logll = self.log.lock().await;
        // let msg = logll.read_latest()?;
        Ok(Acker {
            msg_id: ans.id,
            data: ans.content.clone(),
            rawlog: self.log.clone(),
            topic: topic.to_string(),
            is_ok: false,
        })
    }
}
pub struct Message {
    id: u32,
    _topic: String,
    content: String,
}
pub struct Acker<T>
where
    T: log::Log+Send+'static,
{
    msg_id: u32,
    pub data: String,
    rawlog: Arc<Mutex<T>>,
    topic: String,
    is_ok: bool,
}
impl<T> Acker<T>
where
    T: log::Log+Send+'static,
{
    pub async fn ack(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_ok {
            return Ok(());
        }
        self.is_ok = true;
        let mut logll = self.rawlog.lock().await;
        logll.ack(self.msg_id).await?;
        Ok(())
    }
}

pub struct MessageQueueManager<T, B>
where
T: log::Log+Send+'static,
B: log::LogBuilder<T>+Send+'static,
{
    pub queues: Arc<Mutex<HashMap<String, Arc<MessageQueue<T>>>>>,
    pub log_builder: Arc<Mutex<B>>,
}

impl<T, B> MessageQueueManager<T, B>
where
    T: log::Log+Send+'static,
    B: log::LogBuilder<T>+Send+'static,
{
    pub fn new(log_builder: B) -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
            log_builder: Arc::new(Mutex::new(log_builder)),
        }
    }
    pub async fn push(&self, topic: String, content: String) -> Result<(), Box<dyn Error>> {
        let mut queues = self.queues.lock().await;
        let log_builderll = self.log_builder.lock().await;
        let ans = queues
            .entry(topic.clone())
            .or_insert(Arc::new(MessageQueue::new(
                log_builderll.build(topic.as_str())?,
            )));
        drop(log_builderll);
        let ans = ans.clone();
        drop(queues);
        ans.push(Message {
            id: 0,
            _topic: topic,
            content: content,
        })
        .await?;
        Ok(())
    }
    pub async fn read_latest(&self, topic: String) -> Result<Acker<T>, Box<dyn Error>> {
        let mut queues = self.queues.lock().await;
        let log_builderll = self.log_builder.lock().await;
        let _ = queues
            .entry(topic.clone())
            .or_insert(Arc::new(MessageQueue::new(
                log_builderll.build(topic.as_str())?,
            )));
        drop(log_builderll);
        let queue = queues.get(topic.as_str()).unwrap();
        let queue = queue.clone();
        drop(queues);
        queue.read_latest(topic.as_str()).await
    }
}
