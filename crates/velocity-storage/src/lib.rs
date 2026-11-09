use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use velocity_core::{Conversation, InferenceMetrics, Message, Role, VelocityError, VelocityResult};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> VelocityResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| VelocityError::Storage(format!("Failed to open SQLite database: {}", e)))?;

        // Enable Write-Ahead Logging (WAL) and foreign keys for high-performance concurrent reads
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| VelocityError::Storage(format!("Failed to set pragmas: {}", e)))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    pub fn open_in_memory() -> VelocityResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| VelocityError::Storage(format!("Failed to open in-memory SQLite: {}", e)))?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> VelocityResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                model_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                prompt_tokens INTEGER,
                completion_tokens INTEGER,
                ttft_ms REAL,
                tokens_per_sec REAL,
                duration_ms REAL,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at);",
        )
        .map_err(|e| VelocityError::Storage(format!("Failed to initialize schema: {}", e)))?;

        Ok(())
    }

    pub fn save_conversation(&self, conv: &Conversation) -> VelocityResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO conversations (id, title, model_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                model_id = excluded.model_id,
                updated_at = excluded.updated_at",
            params![
                conv.id.to_string(),
                conv.title,
                conv.model_id,
                conv.created_at.to_rfc3339(),
                conv.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| VelocityError::Storage(format!("Failed to save conversation: {}", e)))?;

        Ok(())
    }

    pub fn list_conversations(&self) -> VelocityResult<Vec<Conversation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, title, model_id, created_at, updated_at FROM conversations ORDER BY updated_at DESC")
            .map_err(|e| VelocityError::Storage(format!("Query prep error: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let title: String = row.get(1)?;
                let model_id: String = row.get(2)?;
                let created_str: String = row.get(3)?;
                let updated_str: String = row.get(4)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_default();
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Conversation {
                    id,
                    title,
                    model_id,
                    created_at,
                    updated_at,
                })
            })
            .map_err(|e| VelocityError::Storage(format!("Query map error: {}", e)))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(c) = r {
                list.push(c);
            }
        }
        Ok(list)
    }

    pub fn delete_conversation(&self, id: Uuid) -> VelocityResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM conversations WHERE id = ?1", params![id.to_string()])
            .map_err(|e| VelocityError::Storage(format!("Failed to delete conversation: {}", e)))?;
        Ok(())
    }

    pub fn save_message(&self, msg: &Message) -> VelocityResult<()> {
        let conn = self.conn.lock().unwrap();
        let role_str = match msg.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };

        let (prompt_tok, comp_tok, ttft, tps, dur) = if let Some(m) = &msg.metrics {
            (
                Some(m.prompt_tokens as i64),
                Some(m.completion_tokens as i64),
                Some(m.time_to_first_token_ms),
                Some(m.tokens_per_second),
                Some(m.total_duration_ms),
            )
        } else {
            (None, None, None, None, None)
        };

        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at, prompt_tokens, completion_tokens, ttft_ms, tokens_per_sec, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                msg.id.to_string(),
                msg.conversation_id.to_string(),
                role_str,
                msg.content,
                msg.created_at.to_rfc3339(),
                prompt_tok,
                comp_tok,
                ttft,
                tps,
                dur,
            ],
        )
        .map_err(|e| VelocityError::Storage(format!("Failed to save message: {}", e)))?;

        // Update conversation timestamp
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![msg.created_at.to_rfc3339(), msg.conversation_id.to_string()],
        )
        .map_err(|e| VelocityError::Storage(format!("Failed to update conversation timestamp: {}", e)))?;

        Ok(())
    }

    pub fn list_messages(&self, conversation_id: Uuid) -> VelocityResult<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, role, content, created_at, prompt_tokens, completion_tokens, ttft_ms, tokens_per_sec, duration_ms
                 FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| VelocityError::Storage(format!("Query prep error: {}", e)))?;

        let rows = stmt
            .query_map(params![conversation_id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let role_str: String = row.get(1)?;
                let content: String = row.get(2)?;
                let created_str: String = row.get(3)?;

                let p_tok: Option<i64> = row.get(4)?;
                let c_tok: Option<i64> = row.get(5)?;
                let ttft: Option<f64> = row.get(6)?;
                let tps: Option<f64> = row.get(7)?;
                let dur: Option<f64> = row.get(8)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_default();
                let role = match role_str.as_str() {
                    "system" => Role::System,
                    "user" => Role::User,
                    _ => Role::Assistant,
                };
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let metrics = if let (Some(pt), Some(ct), Some(tt), Some(ts), Some(d)) =
                    (p_tok, c_tok, ttft, tps, dur)
                {
                    Some(InferenceMetrics {
                        prompt_tokens: pt as usize,
                        completion_tokens: ct as usize,
                        time_to_first_token_ms: tt,
                        tokens_per_second: ts,
                        total_duration_ms: d,
                    })
                } else {
                    None
                };

                Ok(Message {
                    id,
                    conversation_id,
                    role,
                    content,
                    created_at,
                    metrics,
                })
            })
            .map_err(|e| VelocityError::Storage(format!("Query map error: {}", e)))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(m) = r {
                list.push(m);
            }
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_crud() {
        let storage = Storage::open_in_memory().expect("in memory sqlite should open");
        let conv = Conversation::new("Test Chat", "test-model");
        storage.save_conversation(&conv).expect("save conversation");

        let list = storage.list_conversations().expect("list conversations");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Test Chat");

        let msg = Message::new(conv.id, Role::User, "Hello world");
        storage.save_message(&msg).expect("save message");

        let msgs = storage.list_messages(conv.id).expect("list messages");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Hello world");
    }
}

