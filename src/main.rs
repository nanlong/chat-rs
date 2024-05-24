use anyhow::Result;
use chat_rs::{util::tracing_init, Message, Peer, State};
use futures::{stream::StreamExt, SinkExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Start Chat server on: {}", addr);

    let app_state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let app_state = Arc::clone(&app_state);

        tokio::spawn(async move {
            if let Err(e) = handle_client(app_state, stream, addr).await {
                info!("Error: {:?}", e)
            }
        });
    }
}

async fn handle_client(app_state: Arc<State>, stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());

    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        _ => return Ok(()),
    };

    let (peer, mut stream_receiver) = app_state.add_peer(addr, username, stream).await;

    user_join(Arc::clone(&app_state), Arc::clone(&peer)).await?;

    while let Some(line) = stream_receiver.next().await {
        info!("Received message: {:?}", line);

        let text = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Error: {:?}", e);
                break;
            }
        };

        let message = Arc::new(Message::chat(peer.username(), text));
        app_state.boardcase(peer.addr(), message).await?;
    }

    user_leave(app_state, peer).await?;

    Ok(())
}

async fn user_join(app_state: Arc<State>, peer: Arc<Peer>) -> Result<()> {
    info!("{} has joined the chat", peer.username());
    let message = Arc::new(Message::joined(peer.username()));
    app_state.boardcase(peer.addr(), message).await?;

    Ok(())
}

async fn user_leave(app_state: Arc<State>, peer: Arc<Peer>) -> Result<()> {
    info!("{} has left the chat", peer.username());
    app_state.remove_peer(peer.addr());

    let message = Arc::new(Message::left(peer.username()));
    app_state.boardcase(peer.addr(), message).await?;

    Ok(())
}
