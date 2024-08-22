use std::sync::Arc;

use tokio::signal::unix::SignalKind;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();
    let ans = args.next().expect("get config path failed");
    let fd = std::fs::OpenOptions::new()
        .read(true)
        .open(ans.as_str())
        .expect("open config file failed");
    let cnf: Config = serde_json::from_reader(fd).expect("parse config content failed");
    let dealy_duration = if let Some(val) = cnf.delay_duration {
        val
    } else {
        0
    };
    let log_builder = easymq::log::FileLogBuilder::new(cnf.log_root.as_str(), dealy_duration);
    let message_manager = easymq::MessageQueueManager::new(log_builder);
    let message_manager = Arc::new(message_manager);
    if let Some(bind_address) = cnf.rest_api_bind {
        tokio::spawn(easymq::rest::run(message_manager, bind_address, None));
    }
    let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("notify interrupt signal failed");
    interrupt.recv().await;
}
#[derive(serde::Deserialize)]
pub struct Config {
    log_root: String,
    delay_duration: Option<u64>,
    rest_api_bind: Option<String>,
}
