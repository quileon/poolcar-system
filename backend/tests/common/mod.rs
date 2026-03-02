use jsonwebtoken::{encode, EncodingKey, Header};
use poolcar_backend::{config::Config, create_app};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username: String,
    role_name: String,
    exp: usize,
}

pub struct TestApp {
    pub address: String,
    token: String,
    client: Client,
    handle: JoinHandle<()>,
}

impl TestApp {
    pub async fn spawn(db_pool: PgPool) -> Self {
        dotenvy::dotenv().ok();
        let config = Config::from_env().unwrap();

        let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
        let redis_pool = redis_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("Failed to create Redis pool");

        let token = create_test_token(&config.jwt_secret);
        let app = create_app(db_pool, redis_pool, None, config);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            address,
            token,
            client: Client::new(),
            handle,
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.client
            .get(format!("{}{}", self.address, path))
            .header("Authorization", format!("Bearer {}", self.token))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.client
            .post(format!("{}{}", self.address, path))
            .header("Authorization", format!("Bearer {}", self.token))
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        self.client
            .put(format!("{}{}", self.address, path))
            .header("Authorization", format!("Bearer {}", self.token))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.client
            .delete(format!("{}{}", self.address, path))
            .header("Authorization", format!("Bearer {}", self.token))
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn create_test_token(jwt_secret: &str) -> String {
    let claims = Claims {
        username: "test".to_string(),
        role_name: "admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Failed to create test token")
}
