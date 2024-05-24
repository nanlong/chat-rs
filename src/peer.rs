use crate::message::Message;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Peer {
    addr: SocketAddr,
    username: String,
    mpsc_sender: mpsc::Sender<Arc<Message>>,
}

impl Peer {
    pub fn new(
        addr: impl Into<SocketAddr>,
        username: impl Into<String>,
        mpsc_sender: mpsc::Sender<Arc<Message>>,
    ) -> Self {
        Peer {
            addr: addr.into(),
            username: username.into(),
            mpsc_sender,
        }
    }

    pub async fn send(&self, message: Arc<Message>) -> anyhow::Result<()> {
        self.mpsc_sender.send(Arc::clone(&message)).await?;
        Ok(())
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }
}
