use std::{
    pin::pin,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_ws::AggregatedMessage;
use futures_util::{
    StreamExt,
    future::{Either, select},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::interval,
};

use crate::{
    commands::{AppDataSync, MessageEvent},
    errors::ErrorDetails,
    state::{
        gis::{
            raster::{AudioTable, RasterMetadata, RenderMethod},
            vector::TouchDeviceVectorOptions,
        },
        touch_device::TouchDeviceSettings,
    },
};

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

    // Ensure the device has the right settings for the current data on start
    app.state::<AppDataSync>().with_project_fallible(|project| {
        for layer in project.get_vectors_for_display() {
            device_sender.send(AppMessage::FetchVector(layer.get_touch_device_info()));
        }

        let band_to_display = project.get_raster_to_display();
        if let Some(mut band) = band_to_display {
            device_sender.send(AppMessage::FetchRaster(dbg!(
                band.get_info_for_display(&app)?
            )));
        }
        Ok(())
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
                        eprintln!("Socket error: {err:?}");
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
    eprintln!("Socket closed: {close_reason:?}");
    device_sender.disconnect();
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum AppMessage {
    FocusBox([f64; 4]),
    FetchRaster(RasterDisplayInfo),
    FetchVector(VectorInfo),
    UpdateVector(VectorInfo),
    RemoveVector(String),
    UpdateGeneralSettings(TouchDeviceSettings),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VectorInfo {
    pub name: String,
    pub settings: TouchDeviceVectorOptions,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RasterDisplayInfo {
    pub src: String,
    pub metadata: RasterMetadata,
    pub audio_table: Option<AudioTable>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RenderDetails {
    RawData {
        src: String,
    },
    #[serde(rename_all = "camelCase")]
    Combined {
        raw_src: String,
        image_src: String,
    },
    Image {
        src: String,
    },
}

impl From<RenderMethod> for RenderDetails {
    fn from(value: RenderMethod) -> Self {
        match value {
            RenderMethod::RawData => RenderDetails::RawData {
                src: "/get_raster".to_string(),
            },
            RenderMethod::Image => RenderDetails::Image {
                src: "/get_image".to_string(),
            },
            RenderMethod::Combined => RenderDetails::Combined {
                raw_src: "/get_raster".to_string(),
                image_src: "/get_image".to_string(),
            },
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum DeviceMessage {
    Data(DeviceData),
    Error(String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceData {
    pub voices: Vec<String>,
}

fn process_device_message(app: AppHandle, message: DeviceMessage) {
    eprintln!("Message: {message:?}");
    match message {
        DeviceMessage::Error(err) => app.state::<AppDataSync>().with_lock(|state| {
            state
                .errors
                .push(ErrorDetails::TouchDeviceError(err).into())
        }),
        _ => {}
    };
    MessageEvent.emit(&app).unwrap();
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

    pub fn disconnect(&self) {
        *self.sender.lock().unwrap() = None
    }

    pub fn connect(&self, sender: UnboundedSender<AppMessage>) {
        eprintln!("Connected");
        *self.sender.lock().unwrap() = Some(sender)
    }
}
