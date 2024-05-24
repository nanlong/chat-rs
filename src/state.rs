use crate::{message::Message, peer::Peer};
use dashmap::DashMap;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::warn;

const MAX_BUFFER_SIZE: usize = 32;

#[derive(Debug, Default)]
pub struct State {
    peers: DashMap<SocketAddr, Arc<Peer>>,
}

impl State {
    pub async fn add_peer(
        &self,
        addr: impl Into<SocketAddr>,
        username: impl Into<String>,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> (Arc<Peer>, SplitStream<Framed<TcpStream, LinesCodec>>) {
        let (sender, mut receiver) = mpsc::channel::<Arc<Message>>(MAX_BUFFER_SIZE);
        let (mut stream_sender, stream_receiver) = stream.split();
        let peer = Arc::new(Peer::new(addr, username, sender));

        self.peers.insert(peer.addr().to_owned(), Arc::clone(&peer));

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Error: {:?}", e);
                    break;
                };
            }
        });

        (peer, stream_receiver)
    }

    pub async fn boardcase(&self, addr: &SocketAddr, message: Arc<Message>) -> anyhow::Result<()> {
        for peer in self.peers.iter() {
            let peer = peer.value();

            if peer.addr() == addr {
                continue;
            }

            if let Err(e) = peer.send(Arc::clone(&message)).await {
                warn!("Error: {:?}", e);
                self.peers.remove(peer.addr());
            }
        }

        Ok(())
    }

    pub fn remove_peer(&self, addr: &SocketAddr) {
        self.peers.remove(addr);
    }
}
