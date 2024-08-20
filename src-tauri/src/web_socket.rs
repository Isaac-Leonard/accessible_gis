use std::{
    pin::pin,
    time::{Duration, Instant},
};

use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt,
};
use gdal::vector::{LayerAccess, ToGdal};
use itertools::Itertools;
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
                                let message = serde_json::from_str(&text).unwrap();
                                send_msg(app.clone(), &mut session, message).await;
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

async fn send_msg(app: AppHandle, external_display: &mut actix_ws::Session, msg: Message) {
    match msg {
        Message::Screen => external_display.text("message").await.unwrap(),
        Message::App => {
            app.emit("", "Message");
        }
    }
}

enum RecievedMessage {}

fn handle_message() {}

pub struct ExternalTouchDevice<'a> {
    ws_setion: &'a mut actix_ws::Session,
}

impl<'a> ExternalTouchDevice<'a> {
    fn send(polygon: String) {}
}

fn get_current_polygon(point: Point, state: AppState) -> Option<String> {
    // Point is in xy coordinates relative to a screen
    // We need to transform it into coordinates relative to the current geometry layer.
    let point = geo::Point::from(point).to_gdal().unwrap();
    state
        .with_current_vector_layer(|layer| -> Option<String> {
            let feature = layer
                .layer
                .layer
                .features()
                .find(|x| x.geometry().unwrap().contains(&point))?;
            let name = layer.info.primary_field_name.clone();
            Some(match name {
                None => format!("FID: {}", feature.fid().unwrap()),
                Some(name) => {
                    format!(
                        "{}: {}",
                        name,
                        feature.field_as_string_by_name(&name).unwrap().unwrap()
                    )
                }
            })
        })
        .unwrap()
}
