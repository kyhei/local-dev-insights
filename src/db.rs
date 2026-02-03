use anyhow::Result;
use serde::Serialize;
use serde_json;
use sqlx::{FromRow, Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, FromRow)]
pub struct Memo {
    pub id: i64,
    pub content: String,
    pub tags: String,       // stored as JSON string
    pub created_at: String, // Simple string handling for now
}

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(db_url: &str) -> Result<Self> {
        // Ensure the file exists if it's a file path
        if db_url.starts_with("sqlite://") {
            let path_str = &db_url["sqlite://".len()..];
            if !Path::new(path_str).exists() {
                fs::File::create(path_str)?;
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                tags TEXT NOT NULL, -- JSON array of strings
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn list_memos(&self) -> Result<Vec<Memo>> {
        let memos = sqlx::query_as::<_, Memo>(
            "SELECT id, content, tags, created_at FROM memos ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(memos)
    }

    pub async fn add_memo(&self, content: &str, tags: &[String]) -> Result<i64> {
        let tags_json = serde_json::to_string(tags)?;
        let id = sqlx::query("INSERT INTO memos (content, tags) VALUES (?, ?)")
            .bind(content)
            .bind(tags_json)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }
}
