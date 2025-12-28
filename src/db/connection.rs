use rusqlite::{Connection, Result};
use bcrypt::{DEFAULT_COST, hash};
use crate::utils::random::generate_uid;
use std::collections::HashMap;

pub fn initialize_db(conn: &Connection, create_new_db: bool) -> Result<()> {
    if create_new_db == true {
        conn.execute(
        "CREATE TABLE IF NOT EXISTS _configs (
                id INTEGER PRIMARY KEY,
                key VARCHAR(255) NOT NULL,
                value TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "INSERT INTO _configs (key, value) 
            VALUES (?1, ?2)",
            ["secret".to_string(), generate_uid(70)],
        )
        .unwrap();


        conn.execute(
        "CREATE TABLE IF NOT EXISTS _super_admins (
                id INTEGER PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                password VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        [],
        )?;

        let hashed_password =  hash("moosedb", DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO _super_admins (name, email, password) 
            VALUES (?1, ?2, ?3)",
            ["Admin".to_string(), "admin@moosedb".to_string(), hashed_password],
        )
        .unwrap();
    }
    Ok(())
}


pub fn load_configs(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM _configs")?;
    let mut configs = HashMap::new();
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    
    for row_result in rows {
        let (key, value) = row_result?;
        configs.insert(key, value);
    }
    
    Ok(configs)
}