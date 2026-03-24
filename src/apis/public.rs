use crate::AppData;
use crate::Info;

use actix_web::{Error, HttpResponse, Responder, Result, get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<u32>,
    items: Option<u32>,
}

#[derive(Serialize)]
struct RecordsResponse {
    success: bool,
    message: String,
    records: Option<Vec<serde_json::Value>>,
    pagination: Option<PaginationInfo>,
}

#[derive(Serialize)]
struct PaginationInfo {
    current_page: u32,
    items_per_page: u32,
    total_records: i64,
    total_pages: u32,
    records_shown: usize,
    has_next_page: bool,
    has_prev_page: bool,
    next_page: Option<String>,
    prev_page: Option<String>,
}

#[get("/records/{collection_id}")]
pub async fn get_collection_data(
    data: web::Data<AppData>,
    path: web::Path<String>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, Error> {
    let collection_id = path.into_inner();
    let page = query.page.unwrap_or(1).max(1);
    let default_per_page = data
        .configs
        .read()
        .unwrap()
        .get("records_per_page")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(100);
    let items_per_page = query.items.unwrap_or(default_per_page).clamp(1, 10000);

    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse {
                success: false,
                message: format!("Failed to get database connection: {}", err),
                records: None,
                pagination: None,
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(RecordsResponse {
            success: false,
            message: "Metadata table does not exist".to_string(),
            records: None,
            pagination: None,
        }));
    }

    let table_name: Result<String, _> = conn.query_row(
        "SELECT table_name FROM _database_metadata WHERE table_id = ?1 LIMIT 1",
        [&collection_id],
        |row| row.get(0),
    );

    let table_name = match table_name {
        Ok(name) => name,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(HttpResponse::NotFound().json(RecordsResponse {
                success: false,
                message: "Collection not found".to_string(),
                records: None,
                pagination: None,
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse {
                success: false,
                message: format!("Failed to query collection: {}", err),
                records: None,
                pagination: None,
            }));
        }
    };

    let total_records: i64 = match conn.query_row(
        &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
        [],
        |row| row.get(0),
    ) {
        Ok(count) => count,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse {
                success: false,
                message: format!("Failed to count records: {}", err),
                records: None,
                pagination: None,
            }));
        }
    };

    let total_pages = ((total_records as f64) / (items_per_page as f64)).ceil() as u32;
    let offset = (page - 1) * items_per_page;

    let query_sql = format!(
        "SELECT * FROM \"{}\" ORDER BY updated_at DESC LIMIT {} OFFSET {}",
        table_name, items_per_page, offset
    );

    let mut stmt = match conn.prepare(&query_sql) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse {
                success: false,
                message: format!("Failed to prepare query: {}", err),
                records: None,
                pagination: None,
            }));
        }
    };

    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let rows = match stmt.query_map([], |row| {
        let mut record = serde_json::Map::new();
        for i in 0..column_count {
            let value: serde_json::Value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                Ok(rusqlite::types::ValueRef::Integer(v)) => serde_json::json!(v),
                Ok(rusqlite::types::ValueRef::Real(v)) => serde_json::json!(v),
                Ok(rusqlite::types::ValueRef::Text(v)) => {
                    serde_json::Value::String(String::from_utf8_lossy(v).to_string())
                }
                _ => serde_json::Value::Null,
            };
            record.insert(column_names[i].clone(), value);
        }
        Ok(serde_json::Value::Object(record))
    }) {
        Ok(rows) => rows,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(RecordsResponse {
                success: false,
                message: format!("Failed to query records: {}", err),
                records: None,
                pagination: None,
            }));
        }
    };

    let records: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
    let records_shown = records.len();

    let has_next_page = page < total_pages;
    let has_prev_page = page > 1;

    let next_page = if has_next_page {
        Some(format!(
            "/records/{}?page={}&items={}",
            collection_id,
            page + 1,
            items_per_page
        ))
    } else {
        None
    };

    let prev_page = if has_prev_page {
        Some(format!(
            "/records/{}?page={}&items={}",
            collection_id,
            page - 1,
            items_per_page
        ))
    } else {
        None
    };

    let pagination = PaginationInfo {
        current_page: page,
        items_per_page,
        total_records,
        total_pages,
        records_shown,
        has_next_page,
        has_prev_page,
        next_page,
        prev_page,
    };

    Ok(HttpResponse::Ok().json(RecordsResponse {
        success: true,
        message: format!(
            "Retrieved {} of {} total records from '{}' (page {}/{})",
            records_shown, total_records, table_name, page, total_pages
        ),
        records: Some(records),
        pagination: Some(pagination),
    }))
}

#[derive(Serialize)]
struct SingleRecordResponse {
    success: bool,
    message: String,
    record: Option<serde_json::Value>,
}

#[get("/records/{collection_id}/{record_id}")]
pub async fn get_single_record(
    data: web::Data<AppData>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (collection_id, record_id) = path.into_inner();

    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(
                HttpResponse::InternalServerError().json(SingleRecordResponse {
                    success: false,
                    message: format!("Failed to get database connection: {}", err),
                    record: None,
                }),
            );
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(SingleRecordResponse {
            success: false,
            message: "Metadata table does not exist".to_string(),
            record: None,
        }));
    }

    let table_name: Result<String, _> = conn.query_row(
        "SELECT table_name FROM _database_metadata WHERE table_id = ?1 LIMIT 1",
        [&collection_id],
        |row| row.get(0),
    );

    let table_name = match table_name {
        Ok(name) => name,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(HttpResponse::NotFound().json(SingleRecordResponse {
                success: false,
                message: "Collection not found".to_string(),
                record: None,
            }));
        }
        Err(err) => {
            return Ok(
                HttpResponse::InternalServerError().json(SingleRecordResponse {
                    success: false,
                    message: format!("Failed to query collection: {}", err),
                    record: None,
                }),
            );
        }
    };

    let query_sql = format!("SELECT * FROM \"{}\" WHERE id = ?1", table_name);

    let mut stmt = match conn.prepare(&query_sql) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(
                HttpResponse::InternalServerError().json(SingleRecordResponse {
                    success: false,
                    message: format!("Failed to prepare query: {}", err),
                    record: None,
                }),
            );
        }
    };

    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let record_result = stmt.query_row([&record_id], |row| {
        let mut record = serde_json::Map::new();
        for i in 0..column_count {
            let value: serde_json::Value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                Ok(rusqlite::types::ValueRef::Integer(v)) => serde_json::json!(v),
                Ok(rusqlite::types::ValueRef::Real(v)) => serde_json::json!(v),
                Ok(rusqlite::types::ValueRef::Text(v)) => {
                    serde_json::Value::String(String::from_utf8_lossy(v).to_string())
                }
                _ => serde_json::Value::Null,
            };
            record.insert(column_names[i].clone(), value);
        }
        Ok(serde_json::Value::Object(record))
    });

    match record_result {
        Ok(record) => Ok(HttpResponse::Ok().json(SingleRecordResponse {
            success: true,
            message: format!("Record {} retrieved from '{}'", record_id, table_name),
            record: Some(record),
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(HttpResponse::NotFound().json(SingleRecordResponse {
                success: false,
                message: format!("Record '{}' not found in '{}'", record_id, table_name),
                record: None,
            }))
        }
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(SingleRecordResponse {
                success: false,
                message: format!("Failed to query record: {}", err),
                record: None,
            }),
        ),
    }
}

#[post("/get-version")]
pub async fn get_version() -> Result<impl Responder> {
    Ok(web::Json(Info {
        version: 0.1,
        actix_web: 4.0,
        actix_files: 0.6,
        rusqlite: 0.37,
        r2d2: 0.8,
        r2d2_sqlite: 0.31,
        serde: 1.0,
        serde_json: 1.0,
        rust_embed: 8.0,
        mime_guess: 2.0,
    }))
}
