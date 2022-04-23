use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Data {
    pub id: u32,
    pub vec: Vec<u8>,

    #[serde(flatten)]
    pub file_meta: FileMetadata,
}

impl Data {
    pub fn create_empty() -> Data {
        let meta = FileMetadata {filename: "".to_string()};
        return Data{id: 0, vec: Vec::new(), file_meta: meta};
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

