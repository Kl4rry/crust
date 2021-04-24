use std::io::{self, Error, ErrorKind, Read};
use std::result;
use std::str;

//https://stackoverflow.com/questions/50394209/how-can-i-create-an-efficient-iterator-of-chars-from-stdin-with-rust

pub struct Reader<R> {
    inner: R,
}

impl<R: Read> Reader<R> {
    pub fn _new(inner: R) -> Reader<R> {
        Reader { inner }
    }
}

#[derive(Debug)]
pub enum ReaderError {
    NotUtf8,
    Other(Error),
}

impl<R: Read> Iterator for Reader<R> {
    type Item = result::Result<char, ReaderError>;

    fn next(&mut self) -> Option<result::Result<char, ReaderError>> {
        let first_byte = match read_one_byte(&mut self.inner)? {
            Ok(b) => b,
            Err(e) => return Some(Err(ReaderError::Other(e))),
        };
        let width = utf8_char_width(first_byte);
        if width == 1 {
            return Some(Ok(first_byte as char));
        }
        if width == 0 {
            return Some(Err(ReaderError::NotUtf8));
        }
        let mut buf = [first_byte, 0, 0, 0];
        {
            let mut start = 1;
            while start < width {
                match self.inner.read(&mut buf[start..width]) {
                    Ok(0) => return Some(Err(ReaderError::NotUtf8)),
                    Ok(n) => start += n,
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Some(Err(ReaderError::Other(e))),
                }
            }
        }
        Some(match str::from_utf8(&buf[..width]).ok() {
            Some(s) => Ok(s.chars().next().unwrap()),
            None => Err(ReaderError::NotUtf8),
        })
    }
}

fn utf8_char_width(b: u8) -> usize {
    const UTF8_CHAR_WIDTH: [u8; 256] = [
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, // 0x1F
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, // 0x3F
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, // 0x5F
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, // 0x7F
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, // 0x9F
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, // 0xBF
        0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, // 0xDF
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
        4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
    ];

    return UTF8_CHAR_WIDTH[b as usize] as usize;
}

fn read_one_byte(reader: &mut dyn Read) -> Option<io::Result<u8>> {
    let mut buf = [0];
    loop {
        return match reader.read(&mut buf) {
            Ok(0) => None,
            Ok(..) => Some(Ok(buf[0])),
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => Some(Err(e)),
        };
    }
}
