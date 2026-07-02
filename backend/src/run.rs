use crate::loops::mqtt;
use crate::pages;
use crate::types;
use rocket::routes;
use rumqttc::Event;
use rumqttc::Packet;
use std::env;
use tracing::{error, info};

pub async fn run_rocket(
    rocket_port: u16,
    db: sea_orm::DatabaseConnection,
    redis: redis::Client,
    tx: tokio::sync::broadcast::Sender<String>,
) -> anyhow::Result<()> {
    let jwt_secret = env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| b"i-love-curren-chan".to_vec());

    let google_api_key =
        env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY environment variable not set");

    let figment = rocket::Config::figment()
        .merge(("port", rocket_port))
        .merge(("address", "0.0.0.0"))
        .merge(("cli_colors", false))
        .merge(("log_level", "normal"));

    let app_config = types::AppConfig {
        jwt_secret,
        google_api_key,
    };

    rocket::custom(figment)
        .manage(db)
        .manage(redis)
        .manage(app_config)
        .manage(tx)
        .mount(
            "/",
            routes![
                pages::login::login,
                pages::login::post_login,
                pages::login::logout,
                pages::api::verify,
                pages::api::api_login,
                pages::api::search_places,
                pages::api::live_ws,
                pages::dashboard::dashboard,
                pages::trackers::list_trackers,
                pages::trackers::create_tracker,
                pages::trackers::update_tracker,
                pages::trackers::delete_tracker,
                pages::cars::list_cars,
                pages::cars::create_car,
                pages::cars::update_car,
                pages::cars::delete_car,
                pages::car_history::list_history,
                pages::car_history::create_history,
                pages::car_history::update_history,
                pages::car_history::delete_history,
                pages::contacts::list_contacts,
                pages::contacts::create_contact,
                pages::contacts::update_contact,
                pages::contacts::delete_contact,
                pages::users::list_users,
                pages::users::create_user,
                pages::users::update_user,
                pages::users::delete_user,
                pages::trips::list_trips,
                pages::trips::create_trip,
                pages::trips::update_trip,
                pages::trips::delete_trip,
                pages::live::live_tracking
            ],
        )
        .mount("/js", rocket::fs::FileServer::from("templates/js"))
        .mount("/css", rocket::fs::FileServer::from("templates/css"))
        .mount("/assets", rocket::fs::FileServer::from("templates/assets"))
        .register("/", rocket::catchers![pages::login::unauthorized])
        .launch()
        .await?;

    Ok(())
}

pub async fn run_mqtt(
    db: sea_orm::DatabaseConnection,
    redis: redis::Client,
    mqtt_host: &str,
    mqtt_port: u16,
    mqtt_client: &str,
    mqtt_use_tls: bool,
    mqtt_username: &str,
    mqtt_password: &str,
    mqtt_topic: &str,
    tx: tokio::sync::broadcast::Sender<String>,
) -> anyhow::Result<()> {
    use rand::RngExt;
    use rumqttc::{AsyncClient, MqttOptions, Transport};
    use std::time::Duration;

    let random_suffix: String = {
        let mut rng = rand::rng();
        (0..8)
            .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
            .collect()
    };
    let mqtt_client_id = format!("{}-{}", mqtt_client, random_suffix);

    let mut mqtt_options = MqttOptions::new(&mqtt_client_id, mqtt_host, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(mqtt_username, mqtt_password);

    if mqtt_use_tls {
        mqtt_options.set_transport(Transport::Tls(rumqttc::TlsConfiguration::Native));
    }

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    client
        .subscribe(mqtt_topic, rumqttc::QoS::AtLeastOnce)
        .await?;

    // Print full MQTT information
    info!("MQTT configured with the following settings:");
    info!("MQTT host: {}", mqtt_host);
    info!("MQTT port: {}", mqtt_port);
    info!("MQTT client ID: {}", mqtt_client_id);
    info!("MQTT topic: {}", mqtt_topic);

    loop {
        match eventloop.poll().await {
            Ok(notification) => match notification {
                Event::Incoming(Packet::ConnAck(_)) => {
                    info!("MQTT has connected successfully!");
                }
                Event::Incoming(Packet::Publish(publish)) => {
                    let topic = publish.topic;
                    info!("Received MQTT payload on topic: {}", &topic);

                    if let Err(e) = mqtt::handle_mqtt_payload(publish.payload, &db, &redis, &tx).await {
                        error!("Error handling MQTT payload on topic {}: {:?}", &topic, e);
                    }
                }
                _ => {}
            },
            Err(e) => {
                error!("Error in MQTT eventloop: {:?}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
