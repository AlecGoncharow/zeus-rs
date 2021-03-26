pub use tokio;

#[allow(dead_code)]
pub mod client;
#[allow(dead_code)]
pub mod connection;
pub mod message;
pub mod server;

use message::{Message, Messageable};
use tokio::sync::oneshot;

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
