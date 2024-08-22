#[cfg(test)]
mod utils {
    use std:: io::Read;

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
    fn raw(){
        let mut fd=std::fs::OpenOptions::new().read(true).open("./test1").expect("open file failed");
        let mut buff=Vec::new();
        fd.read_to_end(&mut buff).expect("read failed");
        println!("{:?}",buff);

    }
}
