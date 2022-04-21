use std::io::{self, Write};
use std::error::Error;

use crate::Client;
use crate::key::Key;

pub fn console(client : Box<Client>) {
    // Kick it off.
    loop {
        let mut line = String::new();
        print!(">: ");
        io::stdout().flush();

        io::stdin().read_line(&mut line).unwrap();
        
        handle_input_line(client.clone(), line.clone());
    }
}




fn handle_input_line(mut client: Box<Client>, line: String) -> Result<(), Box<dyn Error>>  {
    let mut args = line.split(' ');
    let cmd = args.next().unwrap();
    let cmd = cmd.trim();      
    println!("CMD: {}", cmd);
    match cmd {
        "PTEST" => {
            client.get_peer_record();
        },
        "PING" => {
            let key = args.next().unwrap().trim();
            let parse_key: u32 = key.parse::<u32>().expect(key);
            client.ping(Key {key: parse_key});
        },
        "INSERT" => {
            let name = args.next().unwrap().trim();
            let data = args.next().unwrap().trim();
            client.put_data(name.to_string(), data.to_string());
        },
        "GET" => {
            let key = args.next().unwrap().trim();
            let parse_key: u32 = key.parse::<u32>().expect(key);

            client.get_data(Key {key: parse_key});
        }, "LIST" => {
            client.print_state();
        }, "PROVIDERS" => {
            client.get_providers();
        },
        _ => {
        },
    }
    return Ok(());
}

