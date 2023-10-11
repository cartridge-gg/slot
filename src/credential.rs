use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub token_type: String,
}

impl Credentials {
    pub fn load() -> io::Result<Self> {
        let mut path = dirs::config_local_dir().unwrap();
        path.push("slot/credentials.json");
        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let credentials: Credentials = serde_json::from_str(&contents)?;
        Ok(credentials)
    }

    pub fn write(&self) -> io::Result<()> {
        let mut path = dirs::config_local_dir().unwrap();
        path.push("slot/credentials.json");
        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = OpenOptions::new().write(true).create(true).open(&path)?;
        let serialized = serde_json::to_string(self)?;
        file.write_all(serialized.as_bytes())
    }
}
