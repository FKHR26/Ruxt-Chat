use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct FriendEntry {
    pub conversation_id: Uuid,
    pub friend_username: String,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct DmMessage {
    pub id: Uuid,
    pub sender_username: String,
    pub content: String,
}

/// Holds the DB pool plus one in-memory broadcast sender per conversation,
/// used to fan live messages out to currently-connected WebSocket clients.
/// All durable data (users, friends, conversations, messages) lives in
/// Postgres; this struct only adds the live "who's listening right now"
/// layer on top.
pub struct Registry {
    pub pool: PgPool,
    pub conversation_txs: Mutex<HashMap<Uuid, broadcast::Sender<String>>>,
}

impl Registry {
    pub async fn new(pool: PgPool) -> Self {
        let registry = Registry {
            pool,
            conversation_txs: Mutex::new(HashMap::new()),
        };
        registry.load_existing_conversations().await;
        registry
    }

    async fn load_existing_conversations(&self) {
        let ids: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM conversations")
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();
        let mut txs = self.conversation_txs.lock().unwrap();
        for (id,) in ids {
            txs.entry(id)
                .or_insert_with(|| broadcast::channel::<String>(100).0);
        }
    }

    /// Orders two user ids consistently so (A,B) and (B,A) map to the same pair.
    fn ordered_pair(a: Uuid, b: Uuid) -> (Uuid, Uuid) {
        if a < b { (a, b) } else { (b, a) }
    }

    /// Adds `friend_username` as a friend of `user_id`, creating the
    /// friendship both ways and a conversation between them if one doesn't
    /// already exist. Returns None if no user has that username, or if
    /// the username belongs to the requester themselves.
    pub async fn add_friend(&self, user_id: Uuid, friend_username: &str) -> Option<FriendEntry> {
        let friend_row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM users WHERE username = $1")
                .bind(friend_username)
                .fetch_optional(&self.pool)
                .await
                .ok()?;
        let (friend_id,) = friend_row?;
        if friend_id == user_id {
            return None;
        }

        sqlx::query(
            "INSERT INTO friends (user_id, friend_id) VALUES ($1, $2), ($2, $1)
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(friend_id)
        .execute(&self.pool)
        .await
        .ok()?;

        let (user_a, user_b) = Self::ordered_pair(user_id, friend_id);
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM conversations WHERE user_a = $1 AND user_b = $2")
                .bind(user_a)
                .bind(user_b)
                .fetch_optional(&self.pool)
                .await
                .ok()?;

        let conversation_id = match existing {
            Some((id,)) => id,
            None => {
                let id = Uuid::new_v4();
                sqlx::query(
                    "INSERT INTO conversations (id, user_a, user_b) VALUES ($1, $2, $3)",
                )
                .bind(id)
                .bind(user_a)
                .bind(user_b)
                .execute(&self.pool)
                .await
                .ok()?;
                self.conversation_txs
                    .lock()
                    .unwrap()
                    .insert(id, broadcast::channel::<String>(100).0);
                id
            }
        };

        Some(FriendEntry {
            conversation_id,
            friend_username: friend_username.to_string(),
        })
    }

    /// Lists this user's friends, each with the conversation id to open.
    pub async fn list_friends(&self, user_id: Uuid) -> Vec<FriendEntry> {
        sqlx::query_as(
            "SELECT
                c.id as conversation_id,
                u.username as friend_username
             FROM friends f
             JOIN users u ON u.id = f.friend_id
             JOIN conversations c
                ON (c.user_a = LEAST($1, f.friend_id) AND c.user_b = GREATEST($1, f.friend_id))
             WHERE f.user_id = $1
             ORDER BY u.username",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    pub fn sender_for(&self, conversation_id: Uuid) -> Option<broadcast::Sender<String>> {
        self.conversation_txs.lock().unwrap().get(&conversation_id).cloned()
    }

    /// Confirms `user_id` is actually one of the two participants in this
    /// conversation, so a token for one chat can't be used to snoop on another.
    pub async fn user_in_conversation(&self, conversation_id: Uuid, user_id: Uuid) -> bool {
        let row: Option<(Uuid, Uuid)> =
            sqlx::query_as("SELECT user_a, user_b FROM conversations WHERE id = $1")
                .bind(conversation_id)
                .fetch_optional(&self.pool)
                .await
                .unwrap_or(None);
        matches!(row, Some((a, b)) if a == user_id || b == user_id)
    }

    pub async fn save_message(&self, conversation_id: Uuid, sender_id: Uuid, content: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO dm_messages (id, conversation_id, sender_id, content) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(conversation_id)
        .bind(sender_id)
        .bind(content)
        .execute(&self.pool)
        .await
        .expect("insert message failed");
        id
    }

    pub async fn load_history(&self, conversation_id: Uuid) -> Vec<DmMessage> {
        sqlx::query_as(
            "SELECT dm_messages.id, users.username as sender_username, dm_messages.content
             FROM dm_messages
             JOIN users ON users.id = dm_messages.sender_id
             WHERE conversation_id = $1
             ORDER BY created_at ASC LIMIT 200",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }
}