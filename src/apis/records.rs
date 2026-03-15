use crate::AppData;
use crate::Response;
use crate::utils::random::*;

use actix_web::{HttpResponse, Responder, Result, post, web};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
struct FileUpload {
    filename: String,
    mime_type: String,
    data: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CreateRecordRequest {
    collection_id: String,
    data: serde_json::Map<String, serde_json::Value>,
}

fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn save_uploaded_file(upload: &FileUpload) -> Result<String, String> {
    let uploads_dir = Path::new("uploads");
    if !uploads_dir.exists() {
        std::fs::create_dir_all(uploads_dir).map_err(|e| e.to_string())?;
    }

    let stem = Path::new(&upload.filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let ext = Path::new(&upload.filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let slug = slugify(stem);
    let slug = if slug.is_empty() {
        "file".to_string()
    } else {
        slug
    };

    let unique_name = if ext.is_empty() {
        format!("{}-{}", slug, random_numbers(9))
    } else {
        format!("{}-{}.{}", slug, random_numbers(9), ext)
    };

    let file_path = uploads_dir.join(&unique_name);

    let bytes = base64_decode(&upload.data).map_err(|e| format!("Invalid base64: {}", e))?;

    std::fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [0u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }

    let input: Vec<u8> = input.bytes().filter(|&c| c != b'=').collect();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.chunks(4) {
        let b = chunk
            .iter()
            .map(|&c| lookup[c as usize])
            .collect::<Vec<_>>();

        if b.len() >= 2 {
            output.push((b[0] << 2) | (b[1] >> 4));
        }
        if b.len() >= 3 {
            output.push((b[1] << 4) | (b[2] >> 2));
        }
        if b.len() >= 4 {
            output.push((b[2] << 6) | b[3]);
        }
    }

    Ok(output)
}

#[post("/create-record")]
async fn create_record(
    request: web::Json<CreateRecordRequest>,
    app_data: web::Data<AppData>,
) -> Result<impl Responder> {
    let collection_id = request.collection_id.clone();
    let mut data = request.data.clone();

    let conn = match app_data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to get database connection: {}", err),
            }));
        }
    };

    let table_name: Result<String, _> = conn.query_row(
        "SELECT table_name FROM _database_metadata WHERE table_id = ?1 LIMIT 1",
        [&collection_id],
        |row| row.get(0),
    );

    let table_name = match table_name {
        Ok(name) => name,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(HttpResponse::NotFound().json(Response {
                success: false,
                message: format!("Collection with id '{}' not found", collection_id),
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to query collection: {}", err),
            }));
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT field_name, field_type FROM _database_metadata WHERE table_name = ?1 ORDER BY ROWID"
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to prepare metadata query: {}", err),
            }));
        }
    };

    let fields: Result<Vec<(String, String)>, _> = stmt
        .query_map([&table_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .and_then(|mapped_rows| mapped_rows.collect());

    let fields = match fields {
        Ok(fields) => fields,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to fetch field definitions: {}", err),
            }));
        }
    };

    for (field_name, field_type) in &fields {
        if field_type == "FILE" {
            if let Some(value) = data.get(field_name) {
                if value.is_object() {
                    let upload: FileUpload = match serde_json::from_value(value.clone()) {
                        Ok(u) => u,
                        Err(err) => {
                            return Ok(HttpResponse::BadRequest().json(Response {
                                success: false,
                                message: format!("Invalid file data for '{}': {}", field_name, err),
                            }));
                        }
                    };

                    match save_uploaded_file(&upload) {
                        Ok(path) => {
                            data.insert(field_name.clone(), serde_json::Value::String(path));
                        }
                        Err(err) => {
                            return Ok(HttpResponse::InternalServerError().json(Response {
                                success: false,
                                message: format!("Failed to save file '{}': {}", field_name, err),
                            }));
                        }
                    }
                }
            }
        }
    }

    let generated_id = format!("moo{}", simple_uid(12));

    let mut field_names: Vec<String> = vec!["\"id\"".to_string()];
    field_names.extend(fields.iter().map(|(name, _)| format!("\"{}\"", name)));

    let placeholders: Vec<String> = (1..=field_names.len()).map(|i| format!("?{}", i)).collect();

    let insert_sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({})",
        table_name,
        field_names.join(", "),
        placeholders.join(", ")
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(generated_id)];

    params.extend(fields.iter().map(|(field_name, field_type)| {
        let value = data.get(field_name);

        match (value, field_type.as_str()) {
            (Some(v), "INTEGER") if v.is_i64() => {
                Box::new(v.as_i64().unwrap()) as Box<dyn rusqlite::ToSql>
            }
            (Some(v), "BOOLEAN") if v.is_boolean() => {
                Box::new(if v.as_bool().unwrap() { 1i64 } else { 0i64 }) as Box<dyn rusqlite::ToSql>
            }
            (Some(v), "DECIMAL") if v.is_f64() => {
                Box::new(v.as_f64().unwrap()) as Box<dyn rusqlite::ToSql>
            }
            (Some(v), _) if v.is_string() => {
                Box::new(v.as_str().unwrap().to_string()) as Box<dyn rusqlite::ToSql>
            }
            _ => Box::new(rusqlite::types::Null) as Box<dyn rusqlite::ToSql>,
        }
    }));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    if let Err(err) = conn.execute(&insert_sql, params_refs.as_slice()) {
        return Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to insert record: {}", err),
        }));
    }

    Ok(HttpResponse::Ok().json(Response {
        success: true,
        message: format!("Record created successfully in '{}'", table_name),
    }))
}
