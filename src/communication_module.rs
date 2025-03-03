use std::io::{self, Read, Write};
use std::net::TcpStream;

pub fn set_tcp_stream()-> io::Result<TcpStream>{
    
    TcpStream::connect("localhost:8778") 
       
    
}


