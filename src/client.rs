use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
    ptr::read,
};

use serde::Deserialize;
use serde_json::{de::IoRead, value::Serializer, Deserializer};

use crate::{error::Result, GetResponse, KvsError, RemoveResponse, Request, SetResponse};

pub struct Client {
    _addr: SocketAddr,
    writer: BufWriter<TcpStream>,
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
}

impl Client {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let reader = BufReader::new(stream.try_clone()?);
        let deserializer = Deserializer::from_reader(reader);
        let writer = BufWriter::new(stream);
        Ok(Self {
            _addr: addr,
            reader: deserializer,
            writer,
        })
    }

    /// Get the value of a given key from the server.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    /// Set the value of a string key in the server.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set { key, value })?;
        self.writer.flush()?;
        let resp = SetResponse::deserialize(&mut self.reader)?;
        match resp {
            SetResponse::Ok(_) => Ok(()),
            SetResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    /// Remove a string key in the server.
    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Remove { key })?;
        self.writer.flush()?;
        let resp = RemoveResponse::deserialize(&mut self.reader)?;
        match resp {
            RemoveResponse::Ok(_) => Ok(()),
            RemoveResponse::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }
}
