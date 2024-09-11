use std::{
    pin::pin,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::interval,
};

use crate::commands::AppDataSync;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn ws_handle(
    app: AppHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);
    let (app_sender, mut connection_rx) = unbounded_channel::<AppMessage>();
    let device_sender = app.state::<TouchDevice>();
    device_sender.connect(app_sender);
    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);

    app.state::<AppDataSync>().with_lock(|state| {
        let settings = &state.shared.get_raster_to_display()?.info.audio_settings;
        device_sender.send(AppMessage::Gis(GisMessage {
            raster: RasterMessage {
                min_freq: settings.min_freq,
                max_freq: settings.max_freq,
            },
            vector: VectorMessage {},
        }));
        Some(())
    });

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
                                let message = serde_json::from_str::<DeviceMessage>(&text).unwrap();
                                process_device_message(app.clone(), message);
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
                        session
                            .text(serde_json::to_string(&app_msg).unwrap())
                            .await
                            .unwrap();
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
    device_sender.disconnect();
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum AppMessage {
    Gis(GisMessage),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GisMessage {
    pub raster: RasterMessage,
    pub vector: VectorMessage,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RasterMessage {
    pub min_freq: f64,
    pub max_freq: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VectorMessage {}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum DeviceMessage {}

fn process_device_message(_app: AppHandle, message: DeviceMessage) {
    match message {}
}

#[derive(Default)]
pub struct TouchDevice {
    sender: Mutex<Option<UnboundedSender<AppMessage>>>,
}

impl TouchDevice {
    /// Sends a message to the touch device if one is connected
    /// If one is connected return true else return false
    pub fn send(&self, message: AppMessage) -> bool {
        let mut sender = self.sender.lock().unwrap();
        let sent = match &*sender {
            Some(sender) => sender.send(message).is_ok(),
            None => false,
        };
        if !sent {
            *sender = None
        }
        sent
    }

    pub fn is_connected(&self) -> bool {
        self.sender.lock().unwrap().is_some()
    }

    pub fn disconnect(&self) {
        *self.sender.lock().unwrap() = None
    }

    pub fn connect(&self, sender: UnboundedSender<AppMessage>) {
        eprintln!("Connected");
        *self.sender.lock().unwrap() = Some(sender)
    }
}
