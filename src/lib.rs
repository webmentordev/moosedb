use rusqlite::{Connection, Result};
use rand::Rng;
use bcrypt::{DEFAULT_COST, hash};

pub fn initialize_db(conn: &Connection, create_new_db: bool) -> Result<()> {
    if create_new_db == true {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS configurations (
                id INTEGER PRIMARY KEY,
                key VARCHAR(255) NOT NULL,
                value TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
        "CREATE TABLE IF NOT EXISTS super_admins (
                id INTEGER PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                password VARCHAR(255) NOT NULL,
                token TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        [],
        )?;

        let hashed_password =  hash("moosedb", DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO super_admins (name, email, password) 
            VALUES (?1, ?2, ?3)",
            ["Admin".to_string(), "admin@moosedb".to_string(), hashed_password],
        )
        .unwrap();
    }
    Ok(())
}


pub fn generate_uid() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 30;
    let mut rng = rand::rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    password
}