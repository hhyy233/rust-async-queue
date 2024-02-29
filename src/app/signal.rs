use log::info;

use tokio::select;
#[cfg(unix)]
use tokio::signal::unix::{signal, Signal, SignalKind};

#[allow(unused)]
pub enum SigType {
    /// Equivalent to SIGINT on unix systems.
    Interrupt,
    /// Equivalent to SIGTERM on unix systems.
    Terminate,
}

/// The ender listens for signals.
#[cfg(unix)]
pub struct Ender {
    sigint: Signal,
    sigterm: Signal,
}

#[cfg(unix)]
impl Ender {
    pub fn new() -> Result<Self, std::io::Error> {
        let sigint = signal(SignalKind::interrupt())?;
        let sigterm = signal(SignalKind::terminate())?;

        Ok(Ender { sigint, sigterm })
    }

    /// Waits for either an interrupt or terminate.
    pub async fn wait(&mut self) -> Result<SigType, std::io::Error> {
        let sigtype;

        select! {
            _ = self.sigint.recv() => {
                sigtype = SigType::Interrupt
            },
            _ = self.sigterm.recv() => {
                sigtype = SigType::Terminate
            }
        }

        Ok(sigtype)
    }
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
