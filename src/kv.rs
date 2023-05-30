use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::error::{KvsError, Result};

/// kv store
pub struct KvStore {
    db: HashMap<String, String>,
}

/// A k-v store based on memory
impl KvStore {
    /// create memory kv store
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        todo!();
    }

    /// set k-v to memory
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        println!("store.set: {}, {}", key, value);
        self.db.insert(key, value);
        todo!();
    }

    ///
    pub fn get(&self, key: String) -> Result<Option<String>> {
        self.db.get(&key).map(|value| value.to_owned());
        // match self.db.get(&key) {
        //     Some(value) => Some(value.to_owned()),
        //     None => None,
        // };
        todo!();
    }

    ///
    pub fn remove(&mut self, key: String) -> Result<Option<String>> {
        self.db.remove(&key).map(|value| value);
        // match self.db.remove(&key) {
        //     Some(value) => Some(value),
        //     None => None,
        // }
        todo!();
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Start(0))?;
        Ok(Self {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.reader.read(buf)?;
        self.pos += size as u64;
        Ok(size)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<R: Write + Seek> {
    writer: BufWriter<R>,
    pos: u64,
}

impl<R: Write + Seek> BufWriterWithPos<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let pos = match inner.seek(SeekFrom::Start(0)) {
            Ok(pos) => pos,
            Err(e) => return Err(e.into()),
        };
        Ok(Self {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<R: Write + Seek> Write for BufWriterWithPos<R> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = self.writer.write(buf)?;
        self.pos += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

impl<R: Write + Seek> Seek for BufWriterWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        path::{Path, PathBuf},
    };

    use super::Result;
    use super::{BufReaderWithPos, BufWriterWithPos};

    #[test]
    pub fn test_buf_reader_with_pos() -> Result<()> {
        let file = File::open("README.md")?;
        let mut reader = BufReaderWithPos::new(file)?;
        let mut buf = [0u8; 4];
        loop {
            let size = reader.read(&mut buf)?;
            let content = String::from_utf8_lossy(&buf).into_owned();

            if size < 4 {
                break;
            }
        }

        Ok(())
    }

    #[test]
    pub fn test_buf_writer_with_pos() -> Result<()> {
        // let file = File::open("README.md")?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("README2.md")?;
        // let len = file.metadata()?.len();
        let mut writer = BufWriterWithPos::new(file)?;
        // writer.seek(SeekFrom::End(0))?;
        let content = String::from("\n你好世界哈哈");
        let size = writer.write(content.as_bytes())?;
        writer.flush()?;

        Ok(())
    }
}
