use std::net::{SocketAddrV4, Ipv4Addr, TcpListener};

use clap::Parser;

use std::thread;
use crate::data::Data;

use log::{Record, Level, Metadata, LevelFilter, SetLoggerError};

#[path = "./application/console_handle.rs"]
mod console_handle;

#[path = "./application/client.rs"]
mod client;

#[path = "./application/key.rs"]
mod key;

#[path = "./application/data.rs"]
mod data;

#[path = "./connection/connection.rs"]
mod connection;

#[path = "./connection/client_thread.rs"]
mod client_thread;

use crate::client::Client;


static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;
struct ConsoleLogger;

impl log::Log for ConsoleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
     metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}



#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(short)]
    bootnode: bool,
}



fn main() {
    
    std::env::set_var("RUST_BACKTRACE", "1");
    // Initiate Logger
    log::set_logger(&CONSOLE_LOGGER);
    log::set_max_level(LevelFilter::Info);

    // Parse Inputs
    let  cli = Cli::parse();

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
    let addr_string = addr.to_string().clone();
    let mut split = addr_string.split(":");
    split.next();
    let strp = split.next().unwrap();
    port = strp.parse::<u16>().unwrap();

    println!("Server started on {}", port);

    // Run Console and Client Loop
    let host = address.to_string();
    let client = Client::new(host, port.to_string());

    let mut client_thread_copy = client.clone();
    let client_thread = thread::spawn(move || {client_thread_copy.run(listener)});

    let mut client_poll_copy = client.clone();
    let client_poll = thread::spawn(move || {client_poll_copy.poll()});
    
    let console_thread_copy = client.clone();
    let console = thread::spawn(|| console_handle::console(console_thread_copy));

    client_thread.join();
    client_poll.join();
    console.join();
    
}




