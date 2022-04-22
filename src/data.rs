use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileMetadata {
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Data {
    pub id: u32,
    pub vec: Vec<u8>,
    //pub file_meta: Result<FileMetadata>,
}

impl Data {
    pub fn create_empty() -> Data {
        return Data{id: 0, vec: Vec::new()};
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dat = String::from_utf8(self.vec.clone()).expect("");
        write!(f, "{}", dat)
    }
}
impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        return self.id == other.id;
    }
}

