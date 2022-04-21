use std::net::{TcpStream};
use std::io::{Write, BufReader, BufRead};
use std::sync::{Mutex, Arc};
use std::thread;
use rand::Rng;

use crossbeam::channel::unbounded;

use crate::key::Key;
use crate::client::{PeerRecord, parse_peer_record, create_empty_peer_record, DhtType, parse_providers};


pub type ConnectionRef = Arc<Connection>;

#[derive(Clone)]
pub struct DHTMessage {
    pub type_of: String,
    pub sending_node: PeerRecord,
    pub key: PeerRecord,
    pub keys: Vec<PeerRecord>,
    pub data: (Key, DhtType),
    pub providers: Vec<(String, Key)>,
}

#[derive(Clone)]
pub struct Message {
    pub type_of: String,
    pub from: PeerRecord,
    pub to: PeerRecord,
    pub key: PeerRecord,
    pub keys: Vec<PeerRecord>,
    pub data: (Key, DhtType),
    pub providers: Vec<(String, Key)>,
}

impl Message {
    pub fn new(type_of : String, from: PeerRecord, to: PeerRecord, key: PeerRecord, data_key: Key, data: DhtType) -> Message {
        return Message {type_of : type_of, from: from, to: to, key: key, keys: Vec::new(), data: (data_key, data), providers: Vec::new()};
    }
    
    pub fn make_message(&self) -> String {
        if self.type_of == "INIT" {
            let output = format!("P2P/1.0 INIT\r\nFROM- ({},{})\r\nTO- ({},{})\r\n\r\n", self.from.0.key, self.from.1, self.to.0.key, self.to.1);
            return output;
        } else if self.type_of == "PEERS_I" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1);
            return output;
        } else if self.type_of == "PEERS_R" {
            let mut keys = "".to_string();
            for (key, addr) in &self.keys {
                keys +=  &("(".to_string() + &key.key.clone().to_string() + &",".to_string() + addr + &") ".to_string());
            }
            let mut output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\nKEYS- {}", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1, keys);
            output += "\r\n\r\n";
            return output;
        } else if self.type_of == "PROVIDER_GET" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1);
            return output;
        } else if self.type_of == "PROVIDERS_GET_REPLY" {
            let mut providers = "".to_string();
            for (name, key) in &self.providers {
                providers +=  &("(".to_string() + &name.clone().to_string() + &",".to_string() + &key.key.clone().to_string() + &") ".to_string());
            }
            let mut output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\nPROVIDERS-{}\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1, providers);
            output += "\r\n\r\n";
            return output;
        } else if self.type_of == "PING" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1);
            return output;
        } else if self.type_of == "INSERT" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\nPROVIDER-{}\r\nDATA- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1, self.key.1, self.data.0.key, self.data.1);
            return output;
        } else if self.type_of == "PEERS_I_GET" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\nDATA- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1, self.data.0.key, self.data.1);
            return output;
        } else if self.type_of == "PEERS_R_GET" {
            let output = format!("P2P/1.0 {}\r\nFROM- ({},{})\r\nTO- ({},{})\r\nDATA- ({},{})\r\n\r\n", self.type_of, self.from.0.key, self.from.1, self.to.0.key, self.to.1, self.data.0.key, self.data.1);
            return output;
        }
        "".to_string()
    }

    
    pub fn read_message(reader: &mut BufReader<TcpStream>) -> Result<Message, &'static str>  {
        let mut line = String::with_capacity(512);

        let res = reader.read_line(&mut line).unwrap();

        if res == 0 {
            return Err("Error");
        }

        line.split(' ');
        let mut args = line.split(' ');
        args.next().unwrap();

        let type_of : String = args.next().unwrap().to_string();
        let type_of = type_of.trim().to_string();

        println!("---------------------------------------");
        println!("Receieved: {}", type_of);

        let mut from = create_empty_peer_record();
        let mut to = create_empty_peer_record();
        let mut found_key: PeerRecord = (Key{key:0}, "".to_string());
        let mut keys: Vec<PeerRecord> = Vec::new();
        let mut data: (Key, DhtType) = (Key{key:0}, "".to_string());
        let mut providers: Vec<(String, Key)> = Vec::new();
        loop  {
            let mut line = String::with_capacity(512);
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            line.pop();
            line.pop();
            println!("{}", line);
            
            let mut args = line.split('-');
            let key = args.next().ok_or("Error Parsing")?;
            let val = args.next().ok_or("Error Parsing")?;

            if key == "FROM" {
                from = parse_peer_record(val);
            } else if key == "TO" {
                to = parse_peer_record(val);
            } else if key == "KEY" {
                found_key = parse_peer_record(val);
            } else if key == "KEYS" {
                let trimmed = val.trim();
                if trimmed.len() == 0 { continue; }
                let key_list : Vec<&str> = trimmed.split(' ').collect();
                for new_key in key_list {
                    let peer = parse_peer_record(new_key.clone());
                    keys.push(peer);
                }
            } else if key == "DATA" {
                data = parse_peer_record(val);
            } else if key == "PROVIDERS" {
                let trimmed = val.trim();
                if trimmed.len() == 0 { continue; }
                let p_list : Vec<&str> = trimmed.split(' ').collect();
                for provider in p_list {
                    let peer = parse_providers(provider.clone());
                    providers.push(peer);
                }
            } else if key == "PROVIDER" {
                let trimmed = val.trim();
                found_key = (Key {key : 0}, trimmed.clone().to_string());
            }
        }
        println!("---------------------------------------");
        return Ok(Message {type_of : type_of, from: from, to: to, key: found_key, keys: keys, data: data, providers: providers});
    }
}

pub struct Connection {
    pub id: u32,
    pub sender: crossbeam::channel::Sender<Message>,
    pub receiver: crossbeam::channel::Receiver<Message>,
    
    pub send_dht: crossbeam::channel::Sender<DHTMessage>,
    pub recieve_dht: crossbeam::channel::Receiver<DHTMessage>,

    pub finished: Arc<Mutex<bool>>,
}

impl Clone for Connection {
    fn clone(&self) -> Connection {
        return Connection {id: self.id.clone(), sender: self.sender.clone(), receiver: self.receiver.clone(), 
            send_dht: self.send_dht.clone(), recieve_dht: self.recieve_dht.clone(), finished: self.finished.clone()};
    }
}

impl Connection { 
    pub fn new(stream : TcpStream, read: bool, write: bool) -> ConnectionRef {
        let (send_job, recieve_job): (crossbeam::channel::Sender<Message>, crossbeam::channel::Receiver<Message>)= unbounded();
        let (send_dht, recieve_dht): (crossbeam::channel::Sender<DHTMessage>, crossbeam::channel::Receiver<DHTMessage>)= unbounded();

        let mut rng = rand::thread_rng();
        let rand_id = rng.gen::<u32>();
        let conn = Connection {
            id: rand_id,
            sender: send_job, 
            receiver: recieve_job, 
            send_dht: send_dht, 
            recieve_dht: recieve_dht, 
            finished: Arc::new(Mutex::new(false)),
        };
        let console_ptr = Arc::new(conn);
        
        if read {
            let ptr_read = console_ptr.clone();
            let stream_read = stream.try_clone().unwrap();
            thread::spawn(move || {
                match read_thread(stream_read, ptr_read) {
                   Err(_) => {},
                   _ => {} ,
                }
            });
        }
        
        if write {
            let ptr_write= console_ptr.clone();
            let stream_write = stream.try_clone().unwrap();
            thread::spawn(move || {
                write_thread(stream_write, ptr_write)
            });       
        }  
        
        console_ptr
    }
}

fn read_thread(stream: TcpStream, connection: ConnectionRef) -> Result<(), &'static str> {
    let mut reader = BufReader::new(stream);
    loop {

        let msg = Message::read_message(&mut reader)?;

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
                key: msg.key, 
                keys: Vec::new(),
                data: msg.data,
                providers: Vec::new(),
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
                key: create_empty_peer_record(), 
                keys: Vec::new(),
                data: new_msg.data.clone(),
                providers: Vec::new(),
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

fn write_thread(mut stream: TcpStream, connection: ConnectionRef) {
    loop {
        let msg : Message = connection.receiver.recv().unwrap();
        let _res = stream.write(msg.make_message().as_bytes()).unwrap();
        stream.flush().unwrap();
    }
    
}
