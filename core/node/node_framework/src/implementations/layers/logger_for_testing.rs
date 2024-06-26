pub struct Log<'a> {
    file_name: &'a str,
    message: &'a str,
}

impl<'a> Log<'a> {
    pub fn new(file_name: &'a str, message: &'a str) -> Log<'a> {
        Log {
            file_name: file_name,
            message: message,
        }
    }
    pub fn log(&self) {
        println!("file:: {} message:: {}", self.file_name, self.message);
    }
}

#[test]
pub fn test_logger() {
    Log::new("file_name", "message").log();
}
