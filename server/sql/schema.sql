PRAGMA user_version = 1;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS auth_keys (
    id INTEGER PRIMARY KEY,
    auth_key BLOB
);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mb_key_primary INTEGER NOT NULL,
    mb_key_secondary INTEGER,
    message_id INTEGER NOT NULL,
    peer_id INTEGER NOT NULL,
    from_id INTEGER,
    message TEXT NOT NULL,
    date INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mb_metadata (
    mb_key_primary INTEGER NOT NULL,
    mb_key_secondary INTEGER,
    last_message_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mb_pts (
    mb_key_primary INTEGER NOT NULL,
    mb_key_secondary INTEGER,
    user_id INTEGER NOT NULL,
    pts INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT,
    phone TEXT NOT NULL,
    username TEXT
);
