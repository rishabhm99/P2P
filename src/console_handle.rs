use std::io::{self, Write};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::fs::OpenOptions;

use crate::Client;
use crate::key::Key;
use crate::data::Data;
use crate::data::FileMetadata;

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

            let meta = FileMetadata {filename: name.to_string()};
            let insert_data: Data = Data {id: 1, vec: data.to_string().into_bytes(), file_meta: meta};
            client.put_data(name.to_string(), insert_data);
        },
        "GET" => {
            let key = args.next().unwrap().trim();
            let parse_key: u32 = key.parse::<u32>().expect(key);

            let data = client.get_data(Key {key: parse_key}).unwrap();

            let mut file = File::create("./file.txt")?;
            file.write_all(&data.vec);
            
            println!("{:?}", data);
        }, "LIST" => {
            client.print_state();
        }, "PROVIDERS" => {
            client.get_providers();
        }, "UPLOAD" => {
            let filename = args.next().unwrap().trim();
            
            let file = File::open(filename)?;
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;

            let meta = FileMetadata {filename: filename.to_string()};
            let insert_data: Data = Data {id: 1, vec: buffer, file_meta: meta};
            client.put_data(filename.to_string(), insert_data);
        },
        _ => {
        },
    }
    return Ok(());
}

