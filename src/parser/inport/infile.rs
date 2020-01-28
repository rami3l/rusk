use super::InPort;
use std::cell::RefCell;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

pub struct InFile {
    pub file_str: String,
    line: Option<String>,
    reader: RefCell<BufReader<File>>,
}

impl InFile {
    pub fn new(file_str: &str) -> Self {
        Self {
            file_str: file_str.into(),
            line: Some("".into()),
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
    fn line(&self) -> Option<String> {
        self.line.clone()
    }

    fn set_line(&mut self, new_line: Option<String>) {
        self.line = new_line;
    }

    fn read_line(&self) -> Result<Option<String>, Box<dyn Error>> {
        let mut line = String::new();
        match self.reader.borrow_mut().read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => Ok(Some(line)),
            Err(e) => Err(Box::new(e)),
        }
    }
}
