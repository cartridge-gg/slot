use anyhow::Result;
use std::net::SocketAddr;
use urlencoding::encode;

pub struct Browser;

impl Browser {
    pub async fn open(local_addr: &SocketAddr) -> Result<()> {
        let callback_uri = format!("http://{local_addr}/callback").replace("[::1]", "localhost");
        let encoded_callback_uri = encode(&callback_uri);
        let url = format!("https://x.cartridge.gg/slot/auth?callback_uri={encoded_callback_uri}");

        println!("Your browser has been opened to visit: \n\n    {url}\n");
        webbrowser::open(&url)?;

        Ok(())
    }
}
