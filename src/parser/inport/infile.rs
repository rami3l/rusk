use super::InPort;
use std::cell::RefCell;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

pub struct InFile {
    pub file_str: String,
    line: String,
    reader: RefCell<BufReader<File>>,
}

impl InFile {
    pub fn new(file_str: &str) -> Self {
        Self {
            file_str: file_str.into(),
            line: String::new(),
            reader: {
                let file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .open(file_str)
                    .unwrap();
                RefCell::new(BufReader::new(file))
            },
        }
    }
}

impl InPort for InFile {
    fn line(&self) -> String {
        self.line.clone()
    }

    fn set_line(&mut self, new_line: &str) {
        self.line = new_line.into();
    }

    fn read_line(&self) -> Option<Result<String, Box<dyn Error>>> {
        let mut line = String::new();
        match self.reader.borrow_mut().read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(Ok(line)),
            Err(e) => Some(Err(Box::new(e))),
        }
    }
}
