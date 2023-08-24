pub struct Browser;

impl Browser {
    pub async fn open() -> eyre::Result<()> {
        webbrowser::open("https:/x.cartridge.gg/authenticate")?;

        Ok(())
    }
}
