use std::net::SocketAddr;

pub struct Browser;

impl Browser {
    pub async fn open(local_addr: &SocketAddr) -> eyre::Result<()> {
        // TODO:
        //   - client_id?
        //   - state?
        //   - PKCE?
        let url = format!(
            "https:/x.cartridge.gg/auth?redirect_url=http://{redirect_url}",
            redirect_url = local_addr,
        );

        println!("Your browser has been opened to visit: \n\n    {url}");
        webbrowser::open(&url)?;

        Ok(())
    }
}
