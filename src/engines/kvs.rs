use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt::format,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use clap::builder::OsStr;
use crossbeam_skiplist::SkipMap;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::error::{KvsError, Result};

use super::KvsEngine;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in log files. Log files are named after
/// monotonically increasing generation numbers with a `log` extension name.
/// A skip list in memory stores the keys and the value locations for fast query.
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// use kvs::KvsEngine;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct KvStore {
    // directory for the log and other data
    path: Arc<PathBuf>,
    // map generation number to the file reader
    index: Arc<SkipMap<String, CommandPos>>,
    reader: KvReader,
    writer: Arc<Mutex<KvWriter>>,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors during the log replay.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;

        let mut readers = BTreeMap::new();
        let index = Arc::new(SkipMap::new());

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(log_file(&path, gen, false)?)?;
            uncompacted += load_cmd(gen, &mut reader, &*index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = BufWriterWithPos::new(log_file(&path, current_gen, true)?)?;
        let safe_point = Arc::new(AtomicU64::new(0));

        let reader = KvReader {
            path: Arc::clone(&path),
            safe_point,
            readers: RefCell::new(readers),
        };

        let writer = KvWriter {
            reader: reader.clone(),
            writer,
            current_gen,
            uncompacted,
            path: Arc::clone(&path),
            index: Arc::clone(&index),
        };

        Ok(KvStore {
            path,
            reader,
            index,
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

impl KvsEngine for KvStore {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            if let Command::Set { value, .. } = self.reader.read_command(*cmd_pos.value())? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a given key.
    ///
    /// # Error
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}

struct KvReader {
    path: Arc<PathBuf>,
    safe_point: Arc<AtomicU64>,
    readers: RefCell<BTreeMap<u64, BufReaderWithPos<File>>>,
}

impl KvReader {
    fn close_stale_handles(&self) {
        let mut readers = self.readers.borrow_mut();
        while !readers.is_empty() {
            let first_gen = *readers.keys().next().unwrap();
            if self.safe_point.load(Ordering::SeqCst) <= first_gen {
                break;
            }
            readers.remove(&first_gen);
        }
    }

    /// Read the log file at the given `CommandPos`.
    fn read_and<F, R>(&self, cmd_pos: CommandPos, f: F) -> Result<R>
    where
        F: FnOnce(io::Take<&mut BufReaderWithPos<File>>) -> Result<R>,
    {
        self.close_stale_handles();

        let mut readers = self.readers.borrow_mut();
        // Open the file if we haven't opened it in this `KvStoreReader`.
        // We don't use entry API here because we want the errors to be propogated.
        if !readers.contains_key(&cmd_pos.gen) {
            let reader = BufReaderWithPos::new(log_file(&self.path, cmd_pos.gen, false)?)?;
            readers.insert(cmd_pos.gen, reader);
        }
        let reader = readers.get_mut(&cmd_pos.gen).unwrap();
        reader.seek(SeekFrom::Start(cmd_pos.pos))?;
        let cmd_reader = reader.take(cmd_pos.len);
        f(cmd_reader)
    }

    // Read the log file at the given `CommandPos` and deserialize it to `Command`.
    fn read_command(&self, cmd_pos: CommandPos) -> Result<Command> {
        self.read_and(cmd_pos, |cmd_reader| {
            Ok(serde_json::from_reader(cmd_reader)?)
        })
    }
}

impl Clone for KvReader {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            safe_point: self.safe_point.clone(),
            readers: RefCell::new(BTreeMap::new()),
        }
    }
}

struct KvWriter {
    reader: KvReader,
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    // the number of bytes representing "stale" commands that could be
    // deleted during a compaction
    uncompacted: u64,
    path: Arc<PathBuf>,
    index: Arc<SkipMap<String, CommandPos>>,
}

impl KvWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.get(&key) {
                self.uncompacted += old_cmd.value().len;
            }
            self.index
                .insert(key, (self.current_gen, pos..self.writer.pos).into());
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            let pos = self.writer.pos;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("key not found");
                self.uncompacted += old_cmd.value().len;
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                self.uncompacted += self.writer.pos - pos;
            }

            if self.uncompacted > COMPACTION_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    /// Clears stale entries in the log.
    fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = BufWriterWithPos::new(log_file(&self.path, self.current_gen, true)?)?;

        let mut compaction_writer =
            BufWriterWithPos::new(log_file(&self.path, compaction_gen, true)?)?;

        let mut new_pos = 0; // pos in the new log file
        for entry in self.index.iter() {
            let len = self.reader.read_and(*entry.value(), |mut entry_reader| {
                Ok(io::copy(&mut entry_reader, &mut compaction_writer)?)
            })?;
            self.index.insert(
                entry.key().clone(),
                (compaction_gen, new_pos..new_pos + len).into(),
            );
            new_pos += len;
        }
        compaction_writer.flush()?;

        self.reader
            .safe_point
            .store(compaction_gen, Ordering::SeqCst);
        self.reader.close_stale_handles();

        // remove stale log files
        // Note that actually these files are not deleted immediately because `KvStoreReader`s
        // still keep open file handles. When `KvStoreReader` is used next time, it will clear
        // its stale file handles. On Unix, the files will be deleted after all the handles
        // are closed. On Windows, the deletions below will fail and stale files are expected
        // to be deleted in the next compaction.

        let stale_gens = sorted_gen_list(&self.path)?
            .into_iter()
            .filter(|&gen| gen < compaction_gen);
        for stale_gen in stale_gens {
            let file_path = self.path.join(format!("{stale_gen}.x"));
            if let Err(e) = fs::remove_file(&file_path) {
                eprintln!("{:?} cannot be deleted: {}", file_path, e);
            }
        }
        self.uncompacted = 0;

        Ok(())
    }
}

fn log_file(dir: &Path, gen: u64, write: bool) -> io::Result<File> {
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
        let pos = inner.stream_position()?;
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
        let pos = inner.stream_position()?;
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
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Self {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Self {
        Command::Remove { key }
    }
}

/// Represents the position and length of a json-serialized command in the log.
#[derive(Debug, Clone, Copy)]
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
    index: &SkipMap<String, CommandPos>,
) -> Result<u64> {
    // To make sure we read from the beginning of the file.
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0; // number of bytes that can be saved after a compaction.
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                let entry = index.insert(key, (gen, pos..new_pos).into());
                uncompacted += entry.value().len;
            }
            Command::Remove { key } => {
                if let Some(entry) = index.remove(&key) {
                    uncompacted += entry.value().len;
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
