use std::sync::Arc;

use tokio::time;

use crate::{handlers::audit_handler::audit_handler, state::AppState};

pub async fn audit_loop(state: Arc<AppState>) {
    let mut interval = time::interval(time::Duration::from_mins(1));

    loop {
        interval.tick().await;
        tracing::debug!("Started processing audit");
        match audit_handler(state.clone()).await {
            Ok(_) => {
                tracing::debug!("Finished processing audit");
            }
            Err(e) => {
                tracing::error!("Finished processing audit with error: {}", e);
            }
        }
    }
}
