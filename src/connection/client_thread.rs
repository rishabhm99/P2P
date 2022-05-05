use std::net::TcpStream;
use std::io::Write;
use std::io::BufReader;

use crate::client::{PeerRecord, parse_peer_record, create_empty_peer_record, DhtType, parse_providers};
use crate::client::empty_data;
use crate::connection::{Message, ConnectionRef, DHTMessage};
use crate::data::Data;

use log::info;

pub fn read_thread(stream: TcpStream, connection: ConnectionRef) -> Result<(), &'static str> {
    let mut reader = BufReader::new(stream);
    loop {
        let msg = Message::read_message(&mut reader)?;

        let output = format!("RECIEVED: {} FROM- ({},{}) TO- ({},{})",  msg.type_of, msg.from.0.key, msg.from.1, msg.to.0.key, msg.to.1);
        log::info!("{}", output);
        
        if msg.type_of == "INIT" {

        } else if msg.type_of == "PEERS_I" {
            let mut new_msg = msg.clone();
            new_msg.from = msg.to;
            new_msg.to = msg.from.clone();
            new_msg.type_of = "PEERS_R".to_string();
            
            let dht_msg = DHTMessage {
                type_of: "k_peers".to_string(), 
                sending_node: msg.from, 
                key: create_empty_peer_record(), 
                keys: Vec::new(),
                data: new_msg.data.clone(),
                providers: Vec::new(),
                name: "".to_string(),
            };
            connection.send_dht.send(dht_msg);
            let key_msg : DHTMessage = connection.recieve_dht.recv().unwrap();

            new_msg.keys = key_msg.keys.clone();
            new_msg.data = key_msg.data.clone();
            
            connection.sender.send(new_msg);
            
            {
                let mut conn = connection.finished.lock().unwrap();
                *conn = true;
            }
        } else if msg.type_of == "PROVIDER_GET" {
            let mut new_msg = msg.clone();
            new_msg.from = msg.to;
            new_msg.to = msg.from.clone();
            new_msg.type_of = "PROVIDERS_GET_REPLY".to_string();
            
            let dht_msg = DHTMessage {
                type_of: "providers".to_string(), 
                sending_node: msg.from, 
                key: create_empty_peer_record(), 
                keys: Vec::new(),
                data: new_msg.data.clone(),
                providers: Vec::new(),
                name: "".to_string(),
            };
            connection.send_dht.send(dht_msg);
            let key_msg : DHTMessage = connection.recieve_dht.recv().unwrap();

            new_msg.providers = key_msg.providers.clone();
            
            connection.sender.send(new_msg);
            
            {
                let mut conn = connection.finished.lock().unwrap();
                *conn = true;
            }
        } else if msg.type_of == "PING" {
            let mut new_msg = msg.clone();
            new_msg.from = msg.to;
            new_msg.to = msg.from.clone();
            connection.sender.send(new_msg);
        } else if msg.type_of == "INSERT" {
            let dht_msg = DHTMessage {
                type_of: "insert".to_string(), 
                sending_node: msg.from, 
                key: msg.key.clone(), 
                keys: Vec::new(),
                data: msg.data,
                providers: Vec::new(),
                name: msg.key.1.clone(),
            };
            connection.send_dht.send(dht_msg);
        } else if msg.type_of == "PEERS_I_GET" {
            let mut new_msg = msg.clone();
            new_msg.from = msg.to;
            new_msg.to = msg.from.clone();
            new_msg.type_of = "PEERS_R_GET".to_string();
            
            let dht_msg = DHTMessage {
                type_of: "k_peers".to_string(), 
                sending_node: msg.from, 
                key: msg.key.clone(), 
                keys: Vec::new(),
                data: new_msg.data.clone(),
                providers: Vec::new(),
                name: new_msg.data.0.key.to_string(),
            };
            connection.send_dht.send(dht_msg);
            let key_msg : DHTMessage = connection.recieve_dht.recv().unwrap();

            new_msg.keys = key_msg.keys.clone();
            new_msg.data = key_msg.data.clone();
            
            connection.sender.send(new_msg);
            
            {
                let mut conn = connection.finished.lock().unwrap();
                *conn = true;
            }
        }
    } 
}

pub fn write_thread(mut stream: TcpStream, connection: ConnectionRef) {
    loop {
        let msg : Message = connection.receiver.recv().unwrap();
        let _res = stream.write(msg.make_message().as_bytes()).unwrap();
        stream.flush().unwrap();
        
        let output = format!("SENT: {} FROM- ({},{}) TO- ({},{})", msg.type_of, msg.from.0.key, msg.from.1, msg.to.0.key, msg.to.1);
        log::info!("{}", output);
    }
    
}
