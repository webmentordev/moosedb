use crate::utils::random::generate_secret;
use bcrypt::{DEFAULT_COST, hash};
use rusqlite::Error as RusqliteError;
use rusqlite::{Connection, Result, params};
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
            ["secret".to_string(), generate_secret()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO _configs (key, value) 
            VALUES (?1, ?2)",
            ["appname".to_string(), "MooseDB".to_string()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO _configs (key, value) 
            VALUES (?1, ?2)",
            ["records_per_page".to_string(), "100".to_string()],
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

        let hashed_password = hash("moosedb", DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO _super_admins (name, email, password) 
            VALUES (?1, ?2, ?3)",
            [
                "Admin".to_string(),
                "admin@moosedb.com".to_string(),
                hashed_password,
            ],
        )
        .unwrap();
    }
    Ok(())
}

// Load configs
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

// Update super admin
pub fn update_super_user(email: String, new_password: String) -> Result<()> {
    let conn = Connection::open("database.sqlite")?;

    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM _super_admins WHERE email = ?1)",
        params![email],
        |row| row.get(0),
    )?;

    if !exists {
        println!("Super admin with email '{}' not found.", email);
        return Ok(());
    }

    let hashed_password = hash(new_password, DEFAULT_COST).unwrap();
    let updated = conn.execute(
        "UPDATE _super_admins SET password = ?1 WHERE email = ?2",
        params![hashed_password, email],
    )?;

    if updated != 1 {
        eprintln!("Unexpected: {} rows updated", updated);
    }

    Ok(())
}

// Create new super admin
pub fn create_super_admin(
    name: String,
    email: String,
    password: String,
    confirm_password: String,
) -> rusqlite::Result<()> {
    if password != confirm_password {
        return Err(RusqliteError::InvalidParameterName(
            "Passwords do not match".to_string(),
        ));
    }

    let conn = Connection::open("database.sqlite")?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM _super_admins WHERE email = ?1)",
        params![email],
        |row| row.get(0),
    )?;

    if exists {
        return Err(RusqliteError::InvalidParameterName(format!(
            "Super admin with email '{}' already exists",
            email
        )));
    }

    let hashed_password = hash(password, DEFAULT_COST)
        .map_err(|_| RusqliteError::InvalidParameterName("Failed to hash password".to_string()))?;

    conn.execute(
        "INSERT INTO _super_admins (name, email, password) VALUES (?1, ?2, ?3)",
        params![name, email, hashed_password],
    )?;
    Ok(())
}

// Update setting
pub fn update_setting(key: String, new_value: String) -> Result<()> {
    let conn = Connection::open("database.sqlite")?;

    let updated = conn.execute(
        "UPDATE _configs SET value = ?1, updated_at = CURRENT_TIMESTAMP WHERE key = ?2",
        params![new_value, key],
    )?;

    if updated != 1 {
        eprintln!("Unexpected: {} rows updated", updated);
    }
    Ok(())
}

// Update Secret ket
pub fn update_secret_key() -> Result<()> {
    let conn = Connection::open("database.sqlite")?;

    let updated = conn.execute(
        "UPDATE _configs SET value = ?1 WHERE key = 'secret'",
        params![generate_secret()],
    )?;

    if updated != 1 {
        eprintln!(
            "Unexpected: serect key could not be updated. Reason: {}",
            updated
        );
    }
    Ok(())
}
