#![allow(unused)]
use std::io::prelude::*;
use std::net::TcpStream;

pub fn connect_to_server()-> std::io::Result<()>{
    let mut stream = TcpStream::connect("localhost:8778")?;
    let s="p";
    let n = s.len() as u32;
    let bytes = n.to_le_bytes();
    stream.write(&bytes)?;
    stream.write(s.as_bytes())?;
    Ok(())
}