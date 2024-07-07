use rusqlite::{Connection, Error as SqlError};
use std::error::Error;

use crate::time;

pub struct Storage {
    auth_key_storage: Connection,
    users_storage: Connection,
    messages_storage: Connection,
}

pub struct MessageData {
    pub id: i32,
    pub from_id: i64,
    pub chat_id: i64,
    pub text: String,
    pub date: i32,
}

impl Storage {
    pub fn new() -> Self {
        let auth_key_storage = Connection::open("auth_key.db").unwrap();
        let users_storage = Connection::open("users.db").unwrap();
        let messages_storage = Connection::open("messages.db").unwrap();

        auth_key_storage
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS auth_key (
                    auth_key_id INTEGER PRIMARY KEY,
                    auth_key BLOB
                );

                CREATE TABLE IF NOT EXISTS auth_key_sessions (
                    auth_key_id INTEGER,
                    session_id INTEGER,
                    user_id INTEGER,
                    logged_in BOOL
                );",
            )
            .unwrap();

        users_storage
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    first_name TEXT,
                    last_name TEXT,
                    phone_number TEXT,
                    username TEXT
                );",
            )
            .unwrap();

        messages_storage
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS user_messages (
                    id INTEGER,
                    from_id INTEGER,
                    chat_id INTEGER,
                    text TEXT,
                    date INTEGER
                );",
            )
            .unwrap();

        Self {
            auth_key_storage,
            users_storage,
            messages_storage,
        }
    }

    pub fn store_auth_key(
        &mut self,
        auth_key_id: i64,
        auth_key: [u8; 256],
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.auth_key_storage.execute(
            "INSERT INTO auth_key (auth_key_id, auth_key) VALUES (?1, ?2)",
            (&auth_key_id, &auth_key),
        )?;
        Ok(())
    }

    pub fn get_auth_key(
        &self,
        auth_key_id: i64,
    ) -> Result<[u8; 256], Box<dyn Error + Send + Sync>> {
        struct Data([u8; 256]);
        let mut stmt = self
            .auth_key_storage
            .prepare("SELECT auth_key FROM auth_key WHERE auth_key_id = ?1")?;
        let data = stmt.query_row([&auth_key_id], |r| Ok(Data(r.get(0)?)))?;
        Ok(data.0)
    }

    pub fn store_user_message(
        &mut self,
        user_id: i64,
        chat_id: i64,
        text: String,
    ) -> Result<i32, Box<dyn Error + Send + Sync>> {
        self.messages_storage.execute(
            "INSERT INTO user_messages (id, from_id, chat_id, text, date) VALUES (NULL, ?1, ?2, ?3, ?4)",
            (&user_id, &chat_id, &text, time!()),
        )?;
        let mut stmt = self
            .messages_storage
            .prepare("SELECT rowid FROM user_messages ORDER BY ROWID DESC limit 1")?;
        let data = stmt.query_row((), |r| r.get::<_, i32>(0))?;
        Ok(data)
    }

    pub fn get_user_messages(
        &mut self,
        user_id: i64,
        chat_id: i64,
        offset_id: i32,
        limit: i32,
    ) -> Result<Vec<MessageData>, Box<dyn Error + Send + Sync>> {
        let mut stmt = self.messages_storage.prepare(
            "SELECT rowid, from_id, chat_id, text, date FROM user_messages WHERE from_id = ?1 AND chat_id = ?2 AND rowid > ?3 ORDER BY date DESC LIMIT ?4",
        )?;
        let data = stmt
            .query_map((&user_id, &chat_id, &offset_id, &limit), |r| {
                Ok(MessageData {
                    id: r.get(0)?,
                    from_id: r.get(1)?,
                    chat_id: r.get(2)?,
                    text: r.get(3)?,
                    date: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<MessageData>, SqlError>>()?;
        Ok(data)
    }
}
