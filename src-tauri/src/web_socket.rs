use std::{
    pin::pin,
    sync::Arc,
    time::{Duration, Instant},
};

use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        oneshot, Mutex,
    },
    time::interval,
};

pub enum Command {
    Connect {
        conn_tx: UnboundedSender<()>,
        res_tx: oneshot::Sender<bool>,
    },
    Disconnect,
    Message,
}

#[derive(Debug, Clone)]
pub struct WsServerHandle {
    command_tx: Arc<Mutex<Option<UnboundedSender<Command>>>>,
}

impl WsServerHandle {
    pub fn new() -> Self {
        Self {
            command_tx: Default::default(),
        }
    }

    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: UnboundedSender<Command>) -> bool {
        let mut x = self.command_tx.lock().await;
        match *x {
            Some(_) => false,
            None => {
                x.insert(conn_tx);
                true
            }
        }
    }

    pub async fn send(&self, message: ()) -> bool {
        let x = self.command_tx.lock().await;
        x.as_ref()
            .map(|x| x.send(Command::Message).unwrap())
            .is_some()
    }
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn ws_handle(
    server: WsServerHandle,
    app: AppHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);
    let (mut connection_tx, mut connection_rx) = unbounded_channel();
    let conn_id = server.connect(connection_tx).await;
    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);
    let close_reason = loop {
        // most of the futures we process need to be stack-pinned to work with select()

        let tick = pin!(interval.tick());
        let msg_rx = pin!(connection_rx.recv());

        // TODO: nested select is pretty gross for readability on the match
        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            // commands & messages received from client
            Either::Left(left) => match left {
                (Either::Left(left), _) => match left {
                    (Some(Ok(msg)), _) => {
                        match msg {
                            AggregatedMessage::Ping(bytes) => {
                                last_heartbeat = Instant::now();
                                // unwrap:
                                session.pong(&bytes).await.unwrap();
                            }

                            AggregatedMessage::Pong(_) => {
                                last_heartbeat = Instant::now();
                            }

                            AggregatedMessage::Text(text) => {
                                process_text_msg(&server, &mut session, &text).await;
                            }

                            AggregatedMessage::Binary(_bin) => {
                                eprintln!("unexpected binary message");
                            }

                            AggregatedMessage::Close(reason) => break reason,
                        }
                    }

                    // client WebSocket stream error
                    (Some(Err(err)), _) => {
                        break None;
                    }

                    // client WebSocket stream ended
                    (None, _) => break None,
                },
                // Messages received from the application
                (Either::Right(right), _) => match right {
                    (Some(app_msg), _) => {
                        // app_msg is empty for now so just send the empty string
                        session.text("").await.unwrap();
                    }

                    // all connection's message senders were dropped
                    (None, _) => unreachable!(
                    "all connection message senders were dropped; chat server may have panicked"
                ),
                },
            },
            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum Message {
    App,
    Screen,
}

async fn process_text_msg(
    app: &WsServerHandle,
    external_display: &mut actix_ws::Session,
    text: &str,
) {
    let msg = serde_json::from_str::<Message>(text);
    // we check for /<cmd> type of messages
    let msg = match msg {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };
    match msg {
        Message::Screen => external_display.text("message").await.unwrap(),
        Message::App => {
            app.send(());
        }
    }
}