use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct StoredMessage {
    pub id: Uuid,
    pub username: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateServerReq {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateChannelReq {
    pub name: String,
}

/// Holds the DB pool plus one in-memory broadcast sender per channel.
/// Servers/channels/messages themselves now live in Postgres; the
/// broadcast senders are just for fanning live messages out to
/// currently-connected WebSocket clients.
pub struct Registry {
    pub pool: PgPool,
    pub channel_txs: Mutex<HashMap<Uuid, broadcast::Sender<String>>>,
}

impl Registry {
    pub async fn new(pool: PgPool) -> Self {
        let registry = Registry {
            pool,
            channel_txs: Mutex::new(HashMap::new()),
        };
        registry.load_existing_channels().await;
        registry
    }

    /// On startup, create a broadcast sender for every channel already
    /// in the database (so existing channels can receive live messages).
    async fn load_existing_channels(&self) {
        let rows: Vec<Channel> = sqlx::query_as("SELECT id, name FROM channels")
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();
        let mut txs = self.channel_txs.lock().unwrap();
        for row in rows {
            txs.entry(row.id)
                .or_insert_with(|| broadcast::channel::<String>(100).0);
        }
    }

    pub async fn list_servers(&self) -> Vec<Server> {
        let server_rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, name FROM servers ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default();

        let mut servers = Vec::new();
        for (id, name) in server_rows {
            let channels: Vec<Channel> =
                sqlx::query_as("SELECT id, name FROM channels WHERE server_id = $1 ORDER BY name")
                    .bind(id)
                    .fetch_all(&self.pool)
                    .await
                    .unwrap_or_default();
            servers.push(Server { id, name, channels });
        }
        servers
    }

    pub async fn add_server(&self, name: String) -> Server {
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        sqlx::query("INSERT INTO servers (id, name) VALUES ($1, $2)")
            .bind(server_id)
            .bind(&name)
            .execute(&self.pool)
            .await
            .expect("insert server failed");

        sqlx::query("INSERT INTO channels (id, server_id, name) VALUES ($1, $2, 'general')")
            .bind(channel_id)
            .bind(server_id)
            .execute(&self.pool)
            .await
            .expect("insert default channel failed");

        self.channel_txs
            .lock()
            .unwrap()
            .insert(channel_id, broadcast::channel::<String>(100).0);

        Server {
            id: server_id,
            name,
            channels: vec![Channel { id: channel_id, name: "general".into() }],
        }
    }

    pub async fn add_channel(&self, server_id: Uuid, name: String) -> Option<Channel> {
        let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM servers WHERE id = $1")
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await
            .ok()?;
        exists?;

        let channel_id = Uuid::new_v4();
        sqlx::query("INSERT INTO channels (id, server_id, name) VALUES ($1, $2, $3)")
            .bind(channel_id)
            .bind(server_id)
            .bind(&name)
            .execute(&self.pool)
            .await
            .ok()?;

        self.channel_txs
            .lock()
            .unwrap()
            .insert(channel_id, broadcast::channel::<String>(100).0);

        Some(Channel { id: channel_id, name })
    }

    pub fn sender_for(&self, channel_id: Uuid) -> Option<broadcast::Sender<String>> {
        self.channel_txs.lock().unwrap().get(&channel_id).cloned()
    }

    pub async fn save_message(&self, channel_id: Uuid, username: &str, content: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO messages (id, channel_id, username, content) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(channel_id)
        .bind(username)
        .bind(content)
        .execute(&self.pool)
        .await
        .expect("insert message failed");
        id
    }

    pub async fn load_history(&self, channel_id: Uuid) -> Vec<StoredMessage> {
        sqlx::query_as(
            "SELECT id, username, content FROM messages WHERE channel_id = $1 ORDER BY created_at ASC LIMIT 100",
        )
        .bind(channel_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}
