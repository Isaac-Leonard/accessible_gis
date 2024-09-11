use std::{
    pin::pin,
    time::{Duration, Instant},
};

use actix_web::web::Json;
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt,
};
use gdal::vector::{LayerAccess, ToGdal};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::{sync::mpsc::unbounded_channel, time::interval};

use crate::{geometry::Point, state::AppState};

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
    let (_, mut connection_rx) = unbounded_channel::<()>();
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
                        // app_msg is empty for now so just send the empty string
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
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum AppMessage {
    Gis(GisMessage),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GisMessage {
    raster: (),
    vector: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RasterMessage {
    min_freq: f64,
    max_freq: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct VectorMessage {}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum DeviceMessage {}

pub struct ExternalTouchDevice<'a> {
    ws_setion: &'a mut actix_ws::Session,
}

fn process_device_message(_app: AppHandle, message: DeviceMessage) {
    match message {}
}
