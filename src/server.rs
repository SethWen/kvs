use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

use serde_json::{Deserializer, Serializer};

use crate::{error::Result, GetResponse, KvsEngine, RemoveResponse, Request, SetResponse};

pub struct Server {
    engine: Box<dyn KvsEngine>,
    addr: SocketAddr,
}

impl Server {
    pub fn new(engine: Box<dyn KvsEngine>, addr: SocketAddr) -> Self {
        Self { engine, addr }
    }

    pub fn run(&mut self) -> Result<()> {
        println!("run: listening on {}", self.addr);
        let listener = TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.server(stream) {
                        eprintln!("Error on serving client: {}", e);
                    }
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }

        Ok(())
    }

    fn server(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        macro_rules! send_resp {
            ($resp:expr) => {{
                let resp = $resp;
                serde_json::to_writer(&mut writer, &resp)?;
                writer.flush()?;
                println!("Response sent to {}: {:?}", peer_addr, resp);
            }};
        }

        for req in req_reader {
            let req = req?;
            println!("Receive request from {}: {:?}", peer_addr, req);
            match req {
                Request::Get { key } => send_resp!(match self.engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(e) => GetResponse::Err(format!("{}", e)),
                }),
                Request::Set { key, value } => send_resp!(match self.engine.set(key, value) {
                    Ok(_) => SetResponse::Ok(()),
                    Err(e) => SetResponse::Err(format!("{}", e)),
                }),
                Request::Remove { key } => send_resp!(match self.engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok(()),
                    Err(e) => RemoveResponse::Err(format!("{}", e)),
                }),
            }
        }

        Ok(())
    }
}
