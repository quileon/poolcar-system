use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, Packet, QoS};

use crate::AppState;

pub async fn handle_mqtt_loop(state: Arc<AppState>, mqtt_options: rumqttc::MqttOptions) {
    let (mqtt_client, mut mqtt_event_loop) = AsyncClient::new(mqtt_options, 10);

    let mut subscribed = false;

    loop {
        // Loop until we're connected and subscribed
        match mqtt_event_loop.poll().await {
            Ok(notification) => {
                // Subscribe if we're connected and not subscribed yet
                if !subscribed {
                    if let Event::Incoming(Packet::ConnAck(_)) = notification {
                        println!("MQTT Connected! Subscribing to topics...");
                        mqtt_subscribe(&mqtt_client).await;
                        subscribed = true;
                    }
                }

                // Handle incoming messages
                if let Event::Incoming(Packet::Publish(msg)) = notification {
                    println!("Received message on {}: {:?}", msg.topic, msg.payload);
                }
            }
            Err(e) => {
                eprintln!("MQTT Error: {:?}", e);
                subscribed = false;
                tokio::time::sleep(Duration::from_secs(5)).await;
                println!("Attempting to reconnect...");
            }
        }
    }
}

async fn mqtt_subscribe(client: &AsyncClient) {
    match client.subscribe("poolcar/#", QoS::AtMostOnce).await {
        Ok(_) => {
            println!("Successfully subscribed to poolcar/#");
        }
        Err(e) => {
            eprintln!("Failed to subscribe: {:?}", e);
        }
    }
}
