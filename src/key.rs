use farmhash::fingerprint32;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Key {
    pub key: u32
}

impl Key {
    pub fn distance(self, other_key : Key) -> u32 {
        return self.key ^ other_key.key;
    }

    pub fn generate_hash_from_data(data: &String) -> Key {
       let hash = fingerprint32(data.as_bytes());

        return Key{key:hash};
    }
} 
