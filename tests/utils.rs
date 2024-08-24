#[cfg(test)]
mod utils {
    use std::io::Read;

    #[test]
    fn check() {
        let checker = easymq::utils::FileLogChecker::new();
        let ans = checker.check("./test1").expect("check test1 failed");
        let _: Vec<_> = ans
            .into_iter()
            .map(|f| {
                //println!("{:?}", f);
            })
            .collect();
    }
    #[test]
    fn raw() {
        let mut fd = std::fs::OpenOptions::new()
            .read(true)
            .open("./test1")
            .expect("open file failed");
        let mut buff = Vec::new();
        fd.read_to_end(&mut buff).expect("read failed");
        println!("{:?}", buff);
    }
    #[tokio::test]
    async fn read_msg() {
        let mut client = easymq::easymq_protocol::Client::connect("localhost:7777")
            .await
            .expect("connect client failed");
        let content = client.read_latest("test2").await.expect("read msg failed");
        println!("read msg {content}");
    }
    #[tokio::test]
    async fn push() {
        let mut client = easymq::easymq_protocol::Client::connect("localhost:7777")
            .await
            .expect("connect client failed");
        client
            .publish("test2", "hello world")
            .await
            .expect("push msg failed");
    }
}
