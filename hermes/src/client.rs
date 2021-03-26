use crate::connection::Connection;
use crate::message::{Message, Messageable};
use crate::Command;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, mpsc::Sender, oneshot};
use tokio::task;

type ClientResult<T> = Result<T, Box<dyn std::error::Error + Send>>;

pub struct ClientInterface<T: Messageable> {
    messages_in: Arc<Mutex<VecDeque<(std::net::SocketAddr, Message<T>)>>>,
    connection_tx: Sender<Command<T>>,
    connection_handle: task::JoinHandle<()>,
}

impl<T: Messageable> ClientInterface<T> {
    pub fn new() -> Self {
        let messages_in = Arc::new(Mutex::new(VecDeque::new()));
        let (connection_tx, mut cmd_rx) = mpsc::channel::<Command<T>>(32);

        let messages_in_clone = messages_in.clone();
        let connection_handle = tokio::spawn(async move {
            let messages_in = messages_in_clone;
            let mut connection: Connection<T> = Connection::new(messages_in.clone());

            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    Command::Connect { addr, resp } => {
                        println!("trying to connect to {}", addr);
                        let res = connection.connect_to_server(&addr).await;
                        if res.is_ok() {
                            connection.start_read_loop();
                            connection.start_write_loop();
                        }
                        let _ = resp.send(res);
                    }
                    Command::Send { msg, resp } => {
                        connection.send(msg).await;
                        let _ = resp.send(Ok(()));
                    }
                    Command::Ping { resp } => {
                        //connection.ping().await;
                        let _ = resp.send(Ok(()));
                    }
                    Command::IsAlive { resp } => {
                        let _ = resp.send(Ok(connection.is_connected()));
                    }
                }
            }
        });

        Self {
            messages_in,
            connection_handle,
            connection_tx,
        }
    }

    pub async fn connect(&mut self, host: &str, port: u16) -> ClientResult<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Connect {
            addr: format!("{}:{}", host, port),
            resp: resp_tx,
        };

        match self.connection_tx.clone().send(cmd).await {
            Ok(_) => {}
            Err(e) => return Err(Box::new(e)),
        }

        resp_rx.await.expect("client sender dropped")
    }

    pub async fn send(&mut self, msg: Message<T>) -> ClientResult<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Send { msg, resp: resp_tx };

        match self.connection_tx.clone().send(cmd).await {
            Ok(_) => {}
            Err(e) => return Err(Box::new(e)),
        }

        resp_rx.await.expect("client sender dropped")
    }

    pub fn drain_message_queue(&mut self, out: &mut Vec<(std::net::SocketAddr, Message<T>)>) {
        out.extend(self.messages_in.lock().drain(..));
    }

    async fn run(&mut self) {
        todo!()
    }

    pub async fn is_connected(&self) -> ClientResult<bool> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::IsAlive { resp: resp_tx };

        match self.connection_tx.clone().send(cmd).await {
            Ok(_) => {}
            Err(e) => return Err(Box::new(e)),
        }

        resp_rx.await.expect("client sender dropped")
    }
}
