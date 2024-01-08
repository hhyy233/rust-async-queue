use log::info;

#[allow(unused)]
pub enum SigType {
    /// Equivalent to SIGINT on unix systems.
    Interrupt,
    /// Equivalent to SIGTERM on unix systems.
    Terminate,
}

#[cfg(windows)]
pub struct Ender;

#[cfg(windows)]
impl Ender {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Ender)
    }

    pub async fn wait(&mut self) -> Result<SigType, std::io::Error> {
        info!("listening on signal");
        tokio::signal::ctrl_c().await?;

        Ok(SigType::Interrupt)
    }
}
