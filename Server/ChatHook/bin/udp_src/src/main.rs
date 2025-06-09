// Made by Neverless @ BeamMP. Issues? Feel free to ask.
#![allow(non_snake_case)]

use std::{net::{SocketAddr, UdpSocket}, str::FromStr};
use std::env;
use std::io;
use std::io::Read;

use anyhow::{Result, anyhow};

fn main() {
    // ip port dataN
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {panic!("Expected ip port")}

    let ip = args.get(1).unwrap();
    let port = args.get(2).unwrap().parse::<i64>().expect("Expected numerical Port");
    let data;
    if let Some(arg) = args.get(3) { // linux issue
        data = arg.to_string();
    } else {
        data = tryReadStdin().expect("Received no data");
    }

    let socket = getClientSocket().expect("Cannot create UDP socket");
    socket.connect(format!("{}:{}", ip, port)).unwrap();
    socket.send(data.as_bytes()).unwrap();

    //println!("{}", data);
}//

// blocks until data is in stdin and was read til EOF
fn tryReadStdin() -> Result<String> {
    let mut input = Vec::new();
    io::stdin().lock().read_to_end(&mut input)?;
    let input = std::str::from_utf8(&input)?.to_string();
    if input.len() == 0 {return Err(anyhow!("Received Interrupt"))}
    Ok(input)
}

fn getClientSocket() -> Result<UdpSocket, ()> {
    for i in 3400..3500 {
        let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", i)).unwrap();
        if let Ok(socket) = UdpSocket::bind(addr) {
            return Ok(socket);
        }
    }
    Err(())
}