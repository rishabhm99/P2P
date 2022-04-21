use std::fs;
use std::net::{SocketAddrV4, Ipv4Addr, TcpListener, TcpStream};
use std::io::{self, Write, Read, BufReader, BufRead, BufWriter};

use std::error::Error;
use clap::Parser;

use std::thread;

mod client;
mod console_handle;
mod connection;
mod key;

use crate::client::Client;


#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(short)]
    bootnode: bool,
}


fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    // Initiate Logger
    env_logger::init();

    // Parse Inputs
    let mut cli = Cli::parse();
    
    let mut port = if cli.bootnode {
        12345
    } else {
        0
    };
    
    // Listener
    let address = Ipv4Addr::LOCALHOST;
    let socket = SocketAddrV4::new(address, port);
    let listener = TcpListener::bind(socket).unwrap();
    
    // Generate Rand Port
    let addr = listener.local_addr().unwrap();
    let mut addr_string = addr.to_string().clone();
    let mut split = addr_string.split(":");
    split.next();
    let mut strp = split.next().unwrap();
    port = strp.parse::<u16>().unwrap();

    println!("Server started on {}", port);

    // Run Console and Client Loop
    let host = address.to_string();
    let mut client = Client::new(host, port.to_string());

    let mut client_thread_copy = client.clone();
    let client_thread = thread::spawn(move || {client_thread_copy.run(listener)});

    let mut client_poll_copy = client.clone();
    let client_poll = thread::spawn(move || {client_poll_copy.poll()});
    
    let mut console_thread_copy = client.clone();
    let console = thread::spawn(|| console_handle::console(console_thread_copy));

    client_thread.join();
    console.join();
    
}




