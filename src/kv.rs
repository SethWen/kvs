use std::{
    collections::{BTreeMap, HashMap},
    fmt::format,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use clap::builder::OsStr;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// kv store
pub struct KvStore {
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    index: BTreeMap<String, CommandPos>,
    current_gen: u64,
    uncompacted: u64,
}

/// A k-v store based on memory
impl KvStore {
    /// create memory kv store
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        fs::create_dir_all(&path)?;

        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();
        let mut uncompacted = 0u64;

        let gens = sorted_gen_list(&path)?;
        let current_gen = gens.last().unwrap_or(&0) + 1;
        let writer = BufWriterWithPos::new(log_file(&path, current_gen, true)?)?;
        let reader = BufReaderWithPos::new(log_file(&path, current_gen, false)?)?;
        readers.insert(current_gen, reader);

        for gen in gens {
            let mut reader = BufReaderWithPos::new(log_file(&path, gen, false)?)?;
            uncompacted += load_cmd(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }

        Ok(Self {
            path,
            current_gen,
            writer,
            readers,
            index,
            uncompacted,
        })
    }

    /// set k-v to memory
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Set { key, value } = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current_gen, pos..self.writer.pos).into())
            {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    ///
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            if let Some(reader) = self.readers.get_mut(&cmd_pos.gen) {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
                let taker = reader.take(cmd_pos.len);
                if let Command::Set { value, .. } = serde_json::from_reader(taker)? {
                    Ok(Some(value))
                } else {
                    Err(KvsError::UnexpectedCommandType)
                }
            } else {
                Err(KvsError::KeyNotFound)
            }
        } else {
            Ok(None)
        }
    }

    ///
    pub fn remove(&mut self, key: String) -> Result<()> {
        let cmd = Command::rm(key);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Rm { key } = cmd {
            if let Some(value) = self.index.remove(&key) {
                self.uncompacted += value.len;
                Ok(())
            } else {
                Err(KvsError::KeyNotFound)
            }
        } else {
            Err(KvsError::UnexpectedCommandType)
        }
    }

    /// Clears stale entries in the log.
    pub fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file.
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = BufWriterWithPos::new(log_file(&self.path, self.current_gen, true)?)?;

        let mut compaction_writer =
            BufWriterWithPos::new(log_file(&self.path, compaction_gen, true)?)?;

        let mut new_pos = 0; // pos in the new log file.
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self
                .readers
                .get_mut(&cmd_pos.gen)
                .expect("Cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            new_pos += len;
        }
        compaction_writer.flush()?;

        // remove stale log files.
        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();
        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(self.path.join(format!("{stale_gen}.x")))?;
        }
        self.uncompacted = 0;

        Ok(())
    }
}

fn log_file(dir: &PathBuf, gen: u64, write: bool) -> io::Result<File> {
    let file = dir.join(format!("{gen}.x"));
    if write {
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file)
    } else {
        OpenOptions::new().read(true).open(file)
    }
}
/// Returns sorted generation numbers in the given directory.
fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut list: Vec<_> = fs::read_dir(path)?
        .filter_map(|res| {
            if let Ok(dir) = res {
                let path = dir.path();
                if path.is_file() && path.extension() == Some("x".as_ref()) {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .flat_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.trim_end_matches(".x"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    list.sort_unstable();
    Ok(list)
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
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
        let pos = inner.seek(SeekFrom::Current(0))?;
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
        self.writer.flush()
    }
}

impl<R: Write + Seek> Seek for BufWriterWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Rm { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Self {
        Command::Set { key, value }
    }

    fn rm(key: String) -> Self {
        Command::Rm { key }
    }
}

/// Represents the position and length of a json-serialized command in the log.
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        Self {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

fn load_cmd(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    // To make sure we read from the beginning of the file.
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0; // number of bytes that can be saved after a compaction.
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
            }
            Command::Rm { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                // the "remove" command itself can be deleted in the next compaction.
                // so we add its length to `uncompacted`.
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}
