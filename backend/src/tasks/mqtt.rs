use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

use crate::{error::MqttError, handlers::mqtt_handler, AppState};

pub async fn mqtt_loop(state: Arc<AppState>, mqtt_options: MqttOptions) {
    let (mqtt_client, mut mqtt_event_loop) = AsyncClient::new(mqtt_options, 10);
    let mut subscribed = false;

    loop {
        match mqtt_event_loop.poll().await {
            Ok(notification) => {
                if !subscribed {
                    if let Event::Incoming(Packet::ConnAck(_)) = notification {
                        match &mqtt_client.subscribe("poolcar/#", QoS::AtMostOnce).await {
                            Ok(_) => {
                                tracing::info!("MQTT subscribed, started listening");
                                subscribed = true;
                            }
                            Err(e) => {
                                tracing::error!("MQTT failed subscribing: {}", e);
                            }
                        }
                    }
                }

                if let Event::Incoming(Packet::Publish(msg)) = notification {
                    tracing::debug!("Started processing MQTT request topic={}", msg.topic);
                    match mqtt_handler::mqtt_handler(state.clone(), msg.payload).await {
                        Ok(_) => {
                            tracing::debug!("Finished processing MQTT request topic={}", msg.topic)
                        }
                        Err(e @ MqttError::InvalidLocation) => {
                            tracing::warn!(
                                "Finished processing MQTT request with warning topic={}, warning={}",
                                msg.topic,
                                e
                            )
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed processing MQTT request with error topic={}, error={}",
                                msg.topic,
                                e
                            )
                        }
                    };
                }
            }
            Err(e) => {
                tracing::error!("MQTT lost connection: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                subscribed = false;
            }
        }
    }
}
