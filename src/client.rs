use std::net::{SocketAddrV4, Ipv4Addr, TcpListener, TcpStream};
use std::io::{self, Write, Read, BufReader, BufRead, BufWriter};
use std::option;
use std::sync::{Mutex, Arc, Condvar};
use std::sync::RwLock;
use std::sync::mpsc; 
use std::thread;
use std::error::Error;

use crossbeam::channel::unbounded;
use concurrent_queue::ConcurrentQueue;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use priority_queue::PriorityQueue;
use rand::Rng;
use rand::distributions::Uniform;

use crate::connection::{Connection, ConnectionRef};
use crate::connection::Message;
use crate::key::Key;

const BOOTNODES: [&'static  str; 1] = [
    "127.0.0.1:12345"
];

pub type DHT_Type = String;
pub type PeerRecord = (Key, String);

pub fn parse_peer_record(peer_record: &str) -> PeerRecord {
    let val = peer_record.trim();

    let bracket_vals = &val[1..val.len()-1];

    let mut split = bracket_vals.split(',');
    let parse_key = split.next().unwrap().trim();
    let parse_addr = split.next().unwrap().trim();
    let parse_val = parse_key.parse::<u32>().unwrap();
    (Key{key:parse_val}, parse_addr.to_string())
}

pub fn parse_providers(providers: &str) -> (String, Key) {
    let val = providers.trim();

    let bracket_vals = &val[1..val.len()-1];

    let mut split = bracket_vals.split(',');
    let parse_name = split.next().unwrap().trim();
    let parse_key = split.next().unwrap().trim();
    let parse_val = parse_key.parse::<u32>().unwrap();
    (parse_name.to_string(), Key{key:parse_val})
}

pub fn create_empty_peer_record() -> PeerRecord {
    (Key{key:0}, "".to_string())
}

pub fn empty_data(data: &(Key, String)) -> bool {
    if data.0.key == 0 {
        return true;
    }
    return false;
}




const K : i32 = 20;

#[derive(Clone)]
pub struct Client {
    pub host: String,
    pub peer_id: u16,
    pub key : Key,

    pub connections: Arc<Mutex<Vec<ConnectionRef>>>,

    pub providers : Arc<Mutex<HashMap<String, Key>>>,
    pub known_nodes : Arc<Mutex<HashMap<Key, String>>>,
    pub local_hash : Arc<Mutex<HashMap<Key, DHT_Type>>>,
}


impl Client {
    pub fn new(host: String, port: String) -> Box<Client> {
        let mut connections: Vec<ConnectionRef> = vec![];
        let mut known_nodes =  HashMap::new();
        let mut providers =  HashMap::new();

        println!("{} {}", host, port);

        let address = host + ":" + &port;
        let mut is_bootnode = false;
        for boot in BOOTNODES {
            if boot != address {
                let temp_key = Key{key:1};
                known_nodes.insert(temp_key, boot.to_string());
            } else {
                is_bootnode = true;
            }
        }
        
        let new_key = if is_bootnode {
            Key {key: 1}
        } else {
            let mut rng = rand::thread_rng();
            Key {key : rng.gen::<u32>()}
        };

        let client =  Box::new(Client {host: address, peer_id: 0, connections: Arc::new(Mutex::new(connections)), 
                            local_hash : Arc::new(Mutex::new(HashMap::new())), known_nodes: Arc::new(Mutex::new(known_nodes)), 
                            key: new_key, providers: Arc::new(Mutex::new(providers))});

        return client;
    }
    pub fn print_state(self) {
        println!("KNOWN NODES");
        for (key, val) in  self.known_nodes.lock().unwrap().iter() {
            println!(": {} {}", key.key, val);
        }
        println!("DATA");
        for (key, val) in  self.local_hash.lock().unwrap().iter() {
            println!(": {} {}", key.key, val);
        }
        println!("Providers");
        for (key, val) in  self.providers.lock().unwrap().iter() {
            println!(": {} {}", key, val.key);
        }
    }

    pub fn run(&mut self, listener : TcpListener) {
        self.get_peer_record();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let connection = Connection::new(stream, true, true);
            self.connections.lock().unwrap().push(connection);
        }
    }
    
    pub fn reaper(&mut self, mut connection: ConnectionRef) {
        /*
        while true {
            for conn in self.connections.iter() {
                if conn.finished {

                }
            }
        }
        */
    }

    pub fn poll(&mut self) {
        // TEMP FIX NEED THIS FOR SOME REASON: PERTANING TO BLOCKING PROBABLY
        print!("");
        while true {
            let vec = &*self.connections.lock().unwrap();
            for connection in vec {
                let mut msg = match connection.recieve_dht.try_recv() {
                    Ok(msg) => {
                        msg
                    },
                    
                    Err(e)  => continue,
                };


                self.known_nodes.lock().unwrap().insert(msg.sending_node.0.clone(), msg.sending_node.1.clone());
                if msg.type_of == "k_peers" {
                    let mut new_msg = msg.clone();
                    new_msg.keys = self.find_k_closest_computers(&new_msg.key.0.clone());
                    //print!("");
                    let data = new_msg.data.clone();
                    if empty_data(&data) == false {
                        if self.local_hash.lock().unwrap().contains_key(&data.0) {
                            self.providers.lock().unwrap().insert(new_msg.data.1, new_msg.data.0);    
                            let val = self.local_hash.lock().unwrap().get(&new_msg.data.0).unwrap().to_string();
                            new_msg.data.1 = val;

                        }
                     
                    }
                    connection.send_dht.send(new_msg.clone());
                } else if msg.type_of == "insert" {
                    self.local_hash.lock().unwrap().insert(msg.data.0, msg.data.1.clone());
                    self.providers.lock().unwrap().insert(msg.key.1, msg.data.0);        
                } else if msg.type_of == "providers" {
                    let mut new_msg = msg.clone();
                    let mut provider_vector: Vec<(String, Key)> = Vec::new();

                    for (name, key) in self.providers.lock().unwrap().iter() {
                        provider_vector.push(((*name).clone(), (*key).clone()));
                    }
                    new_msg.providers = provider_vector;
                    connection.send_dht.send(new_msg.clone());
                }
            }
        }
    }

    pub fn get_data(&mut self, find_key: Key) -> DHT_Type {
        let comps  = self.find_k_closest_computers(&find_key);

        let mut data : (Key, DHT_Type) = (Key {key: 0}, "".to_string());
        let mut vec: Vec<u32> = Vec::new();
        for (key, address) in comps.clone() {
            if key == self.key {continue;}

            let mut stream = TcpStream::connect(address.clone()).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let peer_record: PeerRecord = (key, address.clone());
            let msg : Message  = Message::new(
                                            "PEERS_I_GET".to_string(), 
                                            (self.key.clone(), self.host.clone()), 
                                            (key.clone(), address.clone()), 
                                            peer_record,
                                            find_key,
                                            "EMPTY".to_string(),
                                        );

            
            
            // SEND TO PEERS REQUESTING K_CLOSEST
            let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
            let id = connection.id.clone();
            {
                connection.sender.send(msg);
            }
            // Recieve K_Closest
            let msg = Message::read_message(&mut reader).unwrap();
            data = (msg.data.0, msg.data.1.clone());
            if data.1 == "EMPTY" {vec.push(1);}
            else { vec.push(0); }
        }

        for (i, (key, address)) in comps.clone().iter().enumerate() {
            if *key == self.key {continue;}
            if vec[i] == 0 { continue; }
            let mut stream = TcpStream::connect(address.clone()).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let peer_record: PeerRecord = (key.clone(), address.clone());
            let msg : Message  = Message::new(
                                            "INSERT".to_string(), 
                                            (self.key.clone(), self.host.clone()), 
                                            (key.clone(), address.clone()), 
                                            peer_record,
                                            data.0.clone(),
                                            data.1.clone(),
                                        );

            
            
            // SEND TO PEERS REQUESTING K_CLOSEST
            let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
            let id = connection.id.clone();
            {
                connection.sender.send(msg);
            }
            // Recieve K_Closest
            let msg = Message::read_message(&mut reader).unwrap();
            data = (msg.data.0, msg.data.1.clone());
        }
        self.local_hash.lock().unwrap().insert(data.0, data.1.clone().to_string());    

        return "".to_string();
    }

    pub fn put_data(&mut self, name: String, file_name : DHT_Type) -> () {
        let calc_key = Key::generate_hash_from_data(&file_name);

        self.local_hash.lock().unwrap().insert(calc_key, file_name.clone());    
        self.providers.lock().unwrap().insert(name.clone(), calc_key.clone());    

        let comps  = self.find_k_closest_computers(&calc_key);
        for (key, address) in comps {
            if key == self.key {continue;}

            let mut stream = TcpStream::connect(address.clone()).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let peer_record: PeerRecord = (Key{key:0}, name.clone());
            let msg : Message  = Message::new(
                                            "INSERT".to_string(), 
                                            (self.key.clone(), self.host.clone()), 
                                            (key.clone(), address.clone()), 
                                            peer_record,
                                            calc_key.clone(),
                                            file_name.clone(),
                                        );

            
            
            // SEND TO PEERS REQUESTING K_CLOSEST
            let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
            let id = connection.id.clone();
            {
                connection.sender.send(msg);
            }
     
            stream.shutdown(std::net::Shutdown::Read);
        }             
    }

    pub fn put_peer_record(&mut self) {

    }

    pub fn get_peer_record(&mut self) {
        let comps  = self.find_k_closest_computers(&self.key);
        for (key, address) in comps {

            let mut stream = TcpStream::connect(address.clone()).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            let peer_record: PeerRecord = (key, address.clone());
            let msg : Message  = Message::new(
                                            "PEERS_I".to_string(), 
                                            (self.key.clone(), self.host.clone()), 
                                            (key.clone(), address.clone()), 
                                            peer_record,
                                            Key{key:0},
                                            "EMPTY".to_string(),
                                        );

            
            
            // SEND TO PEERS REQUESTING K_CLOSEST
            let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
            let id = connection.id.clone();
            {
                connection.sender.send(msg);
            }
            // Recieve K_Closest

            let msg = Message::read_message(&mut reader).unwrap();
            
            for record in msg.keys {
                if record.0 == self.key {continue;}
                if self.known_nodes.lock().unwrap().contains_key(&record.0) == false {
                    self.known_nodes.lock().unwrap().insert(record.0, record.1);    
                }
            }
            
            stream.shutdown(std::net::Shutdown::Read);
        }
    }
    
    pub fn ping(&mut self, key: Key) -> Option<String> {
        let address = self.known_nodes.lock().unwrap().get(&key)?.clone();
        let mut stream = TcpStream::connect(address.clone()).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let peer_record: PeerRecord = (key, address.clone());
        let msg : Message  = Message::new(
                                        "PING".to_string(), 
                                        (self.key.clone(), self.host.clone()), 
                                        (key.clone(), address.clone()), 
                                        create_empty_peer_record(),
                                        Key{key:0},
                                        "EMPTY".to_string(),
                                        );

        
        let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
        let id = connection.id.clone();
        {
            connection.sender.send(msg);
        }
        let msg = Message::read_message(&mut reader).unwrap();

        return Some("Success".to_string());
    }
    
    pub fn get_providers(&mut self) -> Option<DHT_Type> {

        let comps  = self.find_k_closest_computers(&self.key);
        for (key, address) in comps {
            if key == self.key {continue;}

            let mut stream = TcpStream::connect(address.clone()).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let peer_record: PeerRecord = (self.key.clone(), self.host.clone());
            let msg : Message  = Message::new(
                                            "PROVIDER_GET".to_string(), 
                                            (self.key.clone(), self.host.clone()), 
                                            (key.clone(), address.clone()), 
                                            peer_record,
                                            self.key.clone(),
                                            "EMPTY".to_string(),
                                        );

            
            
            // SEND TO PEERS REQUESTING K_CLOSEST
            let connection  = Connection::new(stream.try_clone().unwrap(), false, true);
            let id = connection.id.clone();
            {
                connection.sender.send(msg);
            }
            
            let msg = Message::read_message(&mut reader).unwrap();
            
            for record in msg.providers {
                if self.providers.lock().unwrap().contains_key(&record.0) == false {
                    self.providers.lock().unwrap().insert(record.0, record.1);    
                }
            }

            stream.shutdown(std::net::Shutdown::Read);
        }
                  
        return None;                               
    }

    pub fn find_k_closest_computers(&self, key : &Key) -> Vec<PeerRecord> {             
        let mut k_closest : Vec<PeerRecord> = Vec::new();

        let mut pqueue_distance: PriorityQueue<PeerRecord, u32> = PriorityQueue::new();
        for (curr_key, item) in self.known_nodes.lock().unwrap().iter() {
            pqueue_distance.push((*curr_key, item.to_string()), key.distance(*curr_key));
        }

        for i in 1..K {
            if pqueue_distance.is_empty() { break };

            let (peer_record, val) = pqueue_distance.pop().unwrap(); 
            k_closest.push(peer_record);
        }
        return k_closest;
    }
}





  
