use catte_tl_schema::*;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use sqlx_sqlite::{SqliteQueryResult, SqliteRow};

use crate::{clone_sized_slice, time};

const SCHEMA_VERSION: u32 = 1;
pub struct Storage {
    db: Pool<Sqlite>,
}

impl Storage {
    pub async fn new(path: String) -> Self {
        let db_file = format!("{path}/cattegram.db");
        let db = if !Sqlite::database_exists(&db_file).await.unwrap_or(false) {
            Sqlite::create_database(&db_file).await.unwrap();
            let db = SqlitePool::connect(&db_file).await.unwrap();
            sqlx::query(include_str!("../sql/schema.sql"))
                .execute(&db)
                .await
                .unwrap();
            db
        } else {
            let db = SqlitePool::connect(&db_file).await.unwrap();
            let version: u32 = sqlx::query_scalar("PRAGMA user_version")
                .fetch_one(&db)
                .await
                .unwrap();
            if version != SCHEMA_VERSION {
                panic!("Database version mismatch: {version} != {SCHEMA_VERSION}");
            }
            db
        };
        Self { db }
    }

    pub async fn insert_auth_key(
        &self,
        auth_key_id: i64,
        auth_key: [u8; 256],
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO auth_keys (id, auth_key) VALUES (?, ?)")
            .bind(auth_key_id)
            .bind(&auth_key[..])
            .execute(&self.db)
            .await
    }

    pub async fn get_auth_key(&self, auth_key_id: i64) -> Result<[u8; 256], sqlx::Error> {
        Ok(clone_sized_slice!(
            &sqlx::query_scalar::<_, Vec<u8>>("SELECT auth_key FROM auth_keys WHERE rowid = ?")
                .bind(auth_key_id)
                .fetch_one(&self.db)
                .await?,
            256
        ))
    }

    pub async fn insert_session(
        &self,
        session_id: i64,
        user_id: i64,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO sessions (id, user_id) VALUES (?, ?)")
            .bind(session_id)
            .bind(user_id)
            .execute(&self.db)
            .await
    }

    pub async fn insert_user(
        &self,
        first_name: &str,
        last_name: &str,
        phone: &str,
    ) -> Result<User, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO users (id, first_name, last_name, phone) VALUES (NULL, ?, ?, ?)",
        )
        .bind(first_name)
        .bind(last_name)
        .bind(phone)
        .execute(&self.db)
        .await?;
        self.get_user(result.last_insert_rowid()).await
    }

    pub async fn update_username(&self, user_id: i64, username: &str) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE users SET username = ? WHERE id = ?")
            .bind(username)
            .bind(user_id)
            .execute(&self.db)
            .await
    }

    pub async fn get_user(&self, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .map(Storage::map_user)
            .fetch_one(&self.db)
            .await
    }

    pub async fn get_user_by_phone(&self, phone: &str) -> Result<User, sqlx::Error> {
        sqlx::query("SELECT * FROM users WHERE phone = ?")
            .bind(phone)
            .map(Storage::map_user)
            .fetch_one(&self.db)
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .map(Storage::map_user)
            .fetch_one(&self.db)
            .await
    }

    pub async fn get_user_by_session_id(&self, session_id: i64) -> Result<User, sqlx::Error> {
        let user_id = sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = ?")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        self.get_user(user_id).await
    }

    pub async fn get_users(&self, ids: &[i64]) -> Result<Vec<User>, sqlx::Error> {
        let sql_query = format!(
            "SELECT * FROM users WHERE id IN ({})",
            format!("?{}", ", ?".repeat(ids.len() - 1))
        );
        let mut query = sqlx::query(&sql_query);
        for id in ids {
            query = query.bind(id);
        }
        query.map(Storage::map_user).fetch_all(&self.db).await
    }

    pub async fn get_message_by_global_id(&self, id: i64) -> Result<Message, sqlx::Error> {
        sqlx::query("SELECT * FROM messages WHERE id = ?")
            .bind(id)
            .map(Storage::map_message)
            .fetch_one(&self.db)
            .await
    }

    pub async fn get_messages(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query("SELECT * FROM messages WHERE id > ? AND mb_key_primary = ? AND mb_key_secondary = ? LIMIT ?")
            .bind(offset)
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .bind(limit)
            .map(Storage::map_message)
            .fetch_all(&self.db)
            .await
    }

    pub async fn insert_mb_metadata(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO mb_metadata (mb_key_primary, mb_key_secondary, last_message_id) VALUES (?, ?, 0)")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .execute(&self.db)
            .await
    }

    pub async fn insert_mb_pts(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
        user_id: i64,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO mb_pts (mb_key_primary, mb_key_secondary, user_id, pts) VALUES (?, ?, ?, 0)")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .bind(user_id)
            .execute(&self.db)
            .await
    }

    pub async fn increment_mb_pts(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
        user_id: i64,
        value: i32,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query("UPDATE mb_pts SET pts = pts + ? WHERE mb_key_primary = ? AND mb_key_secondary = ? AND user_id = ?")
            .bind(value)
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .bind(user_id)
            .execute(&self.db)
            .await?;
        self.get_mb_pts(mb_key_primary, mb_key_secondary, user_id)
            .await
    }

    pub async fn get_mb_pts(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
        user_id: i64,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar("SELECT pts FROM mb_pts WHERE mb_key_primary = ? AND mb_key_secondary = ? AND user_id = ?")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .bind(user_id)
            .fetch_one(&self.db)
            .await
    }

    pub async fn get_last_message_id(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT last_message_id FROM mb_metadata WHERE mb_key_primary = ? AND mb_key_secondary = ?")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .fetch_one(&self.db)
            .await
    }

    pub async fn increment_last_message_id(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE mb_metadata SET last_message_id = last_message_id + 1 WHERE mb_key_primary = ? AND mb_key_secondary = ?")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .execute(&self.db)
            .await
    }

    pub async fn insert_message(
        &self,
        mb_key_primary: i64,
        mb_key_secondary: Option<i64>,
        peer_id: i64,
        from_id: Option<i64>,
        message: String,
    ) -> Result<Message, sqlx::Error> {
        let last_message_id = match self
            .get_last_message_id(mb_key_primary, mb_key_secondary)
            .await
        {
            Ok(r) => r,
            Err(_) => {
                self.insert_mb_metadata(mb_key_primary, mb_key_secondary)
                    .await?;
                0
            }
        };
        let result = sqlx::query("INSERT INTO messages (id, mb_key_primary, mb_key_secondary, message_id, peer_id, from_id, message, date) VALUES (NULL, ?, ?, ?, ?, ?, ?, ?)")
            .bind(mb_key_primary)
            .bind(mb_key_secondary)
            .bind(last_message_id + 1)
            .bind(peer_id)
            .bind(from_id)
            .bind(message)
            .bind(time!())
            .execute(&self.db)
            .await?;
        self.increment_last_message_id(mb_key_primary, mb_key_secondary)
            .await?;
        self.get_message_by_global_id(result.last_insert_rowid())
            .await
    }

    pub fn map_user(row: SqliteRow) -> User {
        let mut user = User::default();
        user.id = row.get("id");
        user.first_name = row.get("first_name");
        user.last_name = row.get("last_name");
        user.phone = row.get("phone");
        user.username = row.get("username");
        user
    }

    pub fn map_message(row: SqliteRow) -> Message {
        let mut message = Message::default();
        message.id = row.get("message_id");
        message.message = row.get("message");
        message.date = row.get("date");
        message.peer_id = PeerVariant::PeerUser(Box::new(PeerUser {
            user_id: row.get("peer_id"),
        }));
        message
    }
}
