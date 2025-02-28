use std::fs::File;
use std::io::{BufReader, Read, Result};

struct MyReader<R> {
    reader: R,
    buf: String,
}

impl<R> MyReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::with_capacity(1024),
        }
    }
}

impl<R> MyReader<R>
where
    R: Read,
{
    pub fn process(&mut self) -> Result<usize> {
        self.reader.read_to_string(&mut self.buf)
    }
}


fn main() {
    let file = File::open("F:/Company/PlayerTest.java").unwrap();
    let mut reader = MyReader::new(BufReader::new(file));

    let size = reader.process().unwrap();
    println!("total size read: {}", size);
}