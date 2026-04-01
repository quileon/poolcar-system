use std::sync::Arc;

use tokio::time;

use crate::{handlers::distance_handler::distance_handler, state::AppState};

pub async fn distance_loop(state: Arc<AppState>) {
    let mut interval = time::interval(time::Duration::from_mins(1));

    loop {
        interval.tick().await;
        tracing::debug!("Started processing distances");
        match distance_handler(state.clone()).await {
            Ok(_) => {
                tracing::debug!("Finished processing distances");
            }
            Err(e) => {
                tracing::error!("Finished processing distances with error: {}", e);
            }
        }
    }
}
