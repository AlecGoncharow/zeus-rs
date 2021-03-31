pub use tokio;

#[allow(dead_code)]
pub mod client;
#[allow(dead_code)]
pub mod connection;
pub mod message;
pub mod server;

pub use client::*;
pub use connection::*;
pub use message::*;
pub use server::*;
use tokio::sync::oneshot;

pub type AddressedMessageQueue<T> = std::collections::VecDeque<(std::net::SocketAddr, Message<T>)>;

type Responder<T> = oneshot::Sender<Result<T, Box<dyn std::error::Error + Send>>>;
#[derive(Debug)]
pub enum Command<T: Messageable> {
    Connect {
        addr: String,
        resp: Responder<()>,
    },
    Send {
        msg: Message<T>,
        resp: Responder<()>,
    },
    Ping {
        resp: Responder<()>,
    },
    IsAlive {
        resp: Responder<bool>,
    },
}
