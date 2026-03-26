use crate::AppData;
use crate::Response;
use crate::db::connection::create_super_admin;
use crate::utils::random::*;

use serde::{Deserialize, Serialize};

use actix_web::{Error, HttpResponse, Responder, Result, get, post, web};

#[derive(Deserialize)]
struct CollectionID {
    collection_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ColumnInfo {
    name: String,
    field_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CollectionRecords {
    success: bool,
    message: String,
    records: Option<Vec<serde_json::Value>>,
    columns: Option<Vec<ColumnInfo>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
struct GetCollectionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    collection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    collection_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
struct CollectionData {
    success: bool,
    message: String,
    records: Option<Vec<serde_json::Value>>,
}

#[post("/get-collection")]
pub async fn get_collection(
    data: web::Data<AppData>,
    request: web::Json<GetCollectionRequest>,
) -> Result<impl Responder> {
    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionData {
                success: false,
                message: format!("Failed to get database connection: {}", err),
                records: None,
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(CollectionData {
            success: false,
            message: "Metadata table does not exist".to_string(),
            records: None,
        }));
    }

    let table_name: Result<String, _> = if let Some(collection_id) = &request.collection_id {
        conn.query_row(
            "SELECT table_name FROM _database_metadata WHERE table_id = ?1 LIMIT 1",
            [collection_id],
            |row| row.get(0),
        )
    } else if let Some(collection_name) = &request.collection_name {
        conn.query_row(
            "SELECT table_name FROM _database_metadata WHERE table_name = ?1 LIMIT 1",
            [collection_name],
            |row| row.get(0),
        )
    } else {
        return Ok(HttpResponse::BadRequest().json(CollectionData {
            success: false,
            message: "Either collection_id or collection_name must be provided".to_string(),
            records: None,
        }));
    };

    let table_name = match table_name {
        Ok(name) => name,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(HttpResponse::NotFound().json(CollectionData {
                success: false,
                message: "Collection not found".to_string(),
                records: None,
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionData {
                success: false,
                message: format!("Failed to query collection: {}", err),
                records: None,
            }));
        }
    };

    let mut stmt = match conn.prepare(&format!(
        "SELECT * FROM \"{}\" ORDER BY updated_at DESC",
        table_name
    )) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionData {
                success: false,
                message: format!("Failed to prepare query: {}", err),
                records: None,
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
            return Ok(HttpResponse::InternalServerError().json(CollectionData {
                success: false,
                message: format!("Failed to query records: {}", err),
                records: None,
            }));
        }
    };

    let records: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();

    Ok(HttpResponse::Ok().json(CollectionData {
        success: true,
        message: format!("Retrieved {} records from '{}'", records.len(), table_name),
        records: Some(records),
    }))
}

#[derive(Deserialize)]
struct DeleteCollectionRecords {
    collection_id: String,
    record_ids: Vec<String>,
}

#[derive(Serialize)]
struct DeleteResponse {
    success: bool,
    message: String,
    deleted_count: Option<usize>,
}

#[post("/delete-collection-records")]
pub async fn delete_collection_records(
    data: web::Data<AppData>,
    request: web::Json<DeleteCollectionRecords>,
) -> Result<impl Responder> {
    let collection_id = request.collection_id.clone();
    let record_ids = request.record_ids.clone();

    if record_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DeleteResponse {
            success: false,
            message: "No record IDs provided".to_string(),
            deleted_count: None,
        }));
    }

    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(DeleteResponse {
                success: false,
                message: format!("Failed to get database connection: {}", err),
                deleted_count: None,
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(DeleteResponse {
            success: false,
            message: "Metadata table does not exist".to_string(),
            deleted_count: None,
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
            return Ok(HttpResponse::NotFound().json(DeleteResponse {
                success: false,
                message: format!("Collection with id '{}' not found", collection_id),
                deleted_count: None,
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(DeleteResponse {
                success: false,
                message: format!("Failed to query collection: {}", err),
                deleted_count: None,
            }));
        }
    };

    let mut file_field_stmt = match conn.prepare(
        "SELECT field_name FROM _database_metadata WHERE table_name = ?1 AND field_type = 'FILE'",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(DeleteResponse {
                success: false,
                message: format!("Failed to query file fields: {}", err),
                deleted_count: None,
            }));
        }
    };

    let file_fields: Vec<String> = file_field_stmt
        .query_map([&table_name], |row| row.get(0))
        .and_then(|rows| rows.collect())
        .unwrap_or_default();

    if !file_fields.is_empty() {
        let id_placeholders = record_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let cols = file_fields
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", ");

        let select_query = format!(
            "SELECT {} FROM \"{}\" WHERE id IN ({})",
            cols, table_name, id_placeholders
        );

        let params: Vec<Box<dyn rusqlite::ToSql>> = record_ids
            .iter()
            .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::ToSql>)
            .collect();
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        if let Ok(mut stmt) = conn.prepare(&select_query) {
            let col_count = file_fields.len();
            let _ = stmt
                .query_map(params_refs.as_slice(), |row| {
                    let mut paths = Vec::new();
                    for i in 0..col_count {
                        if let Ok(Some(raw)) = row.get::<_, Option<String>>(i) {
                            if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str(&raw) {
                                for entry in arr {
                                    if let Some(p) = entry.as_str() {
                                        paths.push(p.to_string());
                                    }
                                }
                            } else {
                                paths.push(raw);
                            }
                        }
                    }
                    Ok(paths)
                })
                .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
                .map(|all_paths| {
                    for paths in all_paths {
                        for path in paths {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                });
        }
    }

    let placeholders = record_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "DELETE FROM \"{}\" WHERE id IN ({})",
        table_name, placeholders
    );

    let params: Vec<Box<dyn rusqlite::ToSql>> = record_ids
        .iter()
        .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::ToSql>)
        .collect();

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let deleted_count = match conn.execute(&query, params_refs.as_slice()) {
        Ok(count) => count,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(DeleteResponse {
                success: false,
                message: format!("Failed to delete records: {}", err),
                deleted_count: None,
            }));
        }
    };

    Ok(HttpResponse::Ok().json(DeleteResponse {
        success: true,
        message: format!("Deleted {} record(s) from '{}'", deleted_count, table_name),
        deleted_count: Some(deleted_count),
    }))
}

#[post("/get-collection-records")]
pub async fn get_collection_records(
    data: web::Data<AppData>,
    request: web::Json<CollectionID>,
) -> Result<impl Responder> {
    let collection_id = request.collection_id.clone();

    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to get database connection: {}", err),
                records: None,
                columns: None,
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(CollectionRecords {
            success: false,
            message: "Metadata table does not exist".to_string(),
            records: None,
            columns: None,
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
            return Ok(HttpResponse::NotFound().json(CollectionRecords {
                success: false,
                message: format!("Collection with id '{}' not found", collection_id),
                records: None,
                columns: None,
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to query collection: {}", err),
                records: None,
                columns: None,
            }));
        }
    };

    let mut metadata_stmt = match conn.prepare(
        "SELECT field_name, field_type FROM _database_metadata WHERE table_name = ?1 ORDER BY ROWID"
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to prepare metadata query: {}", err),
                records: None,
                columns: None,
            }));
        }
    };

    let columns_info: Vec<ColumnInfo> = match metadata_stmt.query_map([&table_name], |row| {
        Ok(ColumnInfo {
            name: row.get(0)?,
            field_type: row.get(1)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to query column metadata: {}", err),
                records: None,
                columns: None,
            }));
        }
    };

    let mut all_columns = vec![ColumnInfo {
        name: "id".to_string(),
        field_type: "INTEGER".to_string(),
    }];
    all_columns.extend(columns_info);
    all_columns.push(ColumnInfo {
        name: "created_at".to_string(),
        field_type: "TIMESTAMP".to_string(),
    });
    all_columns.push(ColumnInfo {
        name: "updated_at".to_string(),
        field_type: "TIMESTAMP".to_string(),
    });

    let mut stmt = match conn.prepare(&format!(
        "SELECT * FROM \"{}\" ORDER BY updated_at DESC",
        table_name
    )) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to prepare query: {}", err),
                records: None,
                columns: None,
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
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to query records: {}", err),
                records: None,
                columns: None,
            }));
        }
    };

    let records: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();

    Ok(HttpResponse::Ok().json(CollectionRecords {
        success: true,
        message: format!("Retrieved {} records from '{}'", records.len(), table_name),
        records: Some(records),
        columns: Some(all_columns),
    }))
}

#[post("/delete-collection")]
pub async fn delete_collection(
    data: web::Data<AppData>,
    request: web::Json<CollectionID>,
) -> Result<impl Responder> {
    let collection_id = request.collection_id.clone();

    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to get database connection: {}", err),
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(Response {
            success: false,
            message: "Metadata table does not exist".to_string(),
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

    let file_fields: Vec<String> = conn
        .prepare(
            "SELECT field_name FROM _database_metadata WHERE table_name = ?1 AND field_type = 'FILE'"
        )
        .and_then(|mut stmt| {
            stmt.query_map([&table_name], |row| row.get(0))
                .and_then(|rows| rows.collect())
        })
        .unwrap_or_default();

    if !file_fields.is_empty() {
        let cols = file_fields
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", ");

        let select_query = format!("SELECT {} FROM \"{}\"", cols, table_name);

        if let Ok(mut stmt) = conn.prepare(&select_query) {
            let col_count = file_fields.len();
            let _ = stmt
                .query_map([], |row| {
                    let mut paths = Vec::new();
                    for i in 0..col_count {
                        if let Ok(Some(raw)) = row.get::<_, Option<String>>(i) {
                            if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str(&raw) {
                                for entry in arr {
                                    if let Some(p) = entry.as_str() {
                                        paths.push(p.to_string());
                                    }
                                }
                            } else {
                                paths.push(raw);
                            }
                        }
                    }
                    Ok(paths)
                })
                .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
                .map(|all_paths| {
                    for paths in all_paths {
                        for path in paths {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                });
        }
    }

    if let Err(err) = conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", table_name), []) {
        return Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to drop table: {}", err),
        }));
    }

    if let Err(err) = conn.execute(
        "DELETE FROM _database_metadata WHERE table_id = ?1",
        [&collection_id],
    ) {
        return Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to delete from metadata: {}", err),
        }));
    }

    Ok(HttpResponse::Ok().json(Response {
        success: true,
        message: format!("Collection '{}' deleted successfully", table_name),
    }))
}

#[get("/collections")]
pub async fn get_collections(app_data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let conn = match app_data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to get database connection: {}", err),
            }));
        }
    };
    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );
    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "collections": []
        })));
    }
    let mut stmt =
        match conn.prepare("SELECT DISTINCT table_id, table_name FROM _database_metadata") {
            Ok(stmt) => stmt,
            Err(err) => {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to prepare query: {}", err),
                }));
            }
        };
    let collections: Result<Vec<serde_json::Value>, _> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "table_id": row.get::<_, String>(0)?,
                "table_name": row.get::<_, String>(1)?
            }))
        })
        .and_then(|mapped_rows| mapped_rows.collect());
    match collections {
        Ok(collections) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "collections": collections
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to fetch collections: {}", err),
        })),
    }
}

#[get("/get-super-admins")]
pub async fn get_super_admins(app_data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let conn = match app_data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get database connection: {}", err),
                "records": None::<Vec<serde_json::Value>>,
                "columns": None::<Vec<String>>,
            })));
        }
    };

    let mut stmt = match conn.prepare("SELECT DISTINCT name, email, created_at FROM _super_admins")
    {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to prepare query: {}", err),
                "records": None::<Vec<serde_json::Value>>,
                "columns": None::<Vec<String>>,
            })));
        }
    };

    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let super_admins: Result<Vec<serde_json::Value>, _> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "email": row.get::<_, String>(1)?,
                "created_at": row.get::<_, String>(2)?
            }))
        })
        .and_then(|mapped_rows| mapped_rows.collect());

    match super_admins {
        Ok(super_admins) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "records": super_admins,
            "columns": column_names,
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": format!("Failed to fetch super admins: {}", err),
            "records": None::<Vec<serde_json::Value>>,
            "columns": None::<Vec<String>>,
        }))),
    }
}

#[derive(Deserialize, Serialize)]
struct CreateAdmin {
    name: String,
    email: String,
    password: String,
    confirm_password: String,
}

#[post("/create-super-admin")]
pub async fn create_super_admin_func(request: web::Json<CreateAdmin>) -> Result<impl Responder> {
    match create_super_admin(
        request.name.to_string(),
        request.email.to_string(),
        request.password.to_string(),
        request.confirm_password.to_string(),
    ) {
        Ok(_) => Ok(web::Json(Response {
            success: true,
            message: "Super admin has been added!".to_string(),
        })),
        Err(err) => Ok(web::Json(Response {
            success: false,
            message: err.to_string(),
        })),
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct CollectionRequest {
    collection: String,
    fields: Vec<CollectionFields>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CollectionFields {
    title: String,
    #[serde(rename = "type")]
    field_type: String,
    unique: bool,
    nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    allowed_extensions: Option<String>,
}

#[post("/create-collection")]
pub async fn create_collection(
    data: web::Json<CollectionRequest>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, Error> {
    if data.collection.is_empty() {
        return Ok(HttpResponse::BadRequest().json(Response {
            success: false,
            message: "Collection name is required".to_string(),
        }));
    }

    if data.collection.starts_with('_') {
        return Ok(HttpResponse::BadRequest().json(Response {
            success: false,
            message: "Collection name cannot start with '_'".to_string(),
        }));
    }

    let conn = match app_data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to get database connection: {}", err),
            }));
        }
    };

    let table_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
        [&data.collection],
        |row| row.get(0),
    );

    if let Ok(count) = table_exists {
        if count > 0 {
            return Ok(HttpResponse::BadRequest().json(Response {
                success: false,
                message: format!("Collection {} already exists!", &data.collection),
            }));
        }
    }

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        let create_metadata_table_sql = "CREATE TABLE _database_metadata (
            table_id TEXT NOT NULL,
            table_name TEXT NOT NULL,
            field_name TEXT NOT NULL,
            field_type TEXT NOT NULL,
            unique_field BOOLEAN NOT NULL,
            nullable BOOLEAN NOT NULL,
            min INTEGER,
            max INTEGER,
            allowed_extensions TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (table_name, field_name)
        )";

        if let Err(err) = conn.execute(create_metadata_table_sql, []) {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to create metadata table: {}", err),
            }));
        }
    }

    let table_id = format!("moo_{}", random_numbers(9));

    let mut create_table_sql = format!(
        "CREATE TABLE \"{}\" (id TEXT PRIMARY KEY NOT NULL",
        data.collection
    );

    for field in &data.fields {
        let mut field_def = format!(
            ", \"{}\" {}",
            field.title,
            sql_type_from_field_type(&field.field_type)
        );

        if !field.nullable {
            field_def.push_str(" NOT NULL");
        }
        if field.unique {
            field_def.push_str(" UNIQUE");
        }

        if let Some(min) = field.min {
            if field.field_type == "VARCHAR" || field.field_type == "TEXT" {
                field_def.push_str(&format!(" CHECK(length(\"{}\") >= {})", field.title, min));
            } else if field.field_type == "INTEGER" || field.field_type == "FLOAT" {
                field_def.push_str(&format!(" CHECK(\"{}\" >= {})", field.title, min));
            }
        }

        if let Some(max) = field.max {
            if field.field_type == "VARCHAR" || field.field_type == "TEXT" {
                field_def.push_str(&format!(" CHECK(length(\"{}\") <= {})", field.title, max));
            } else if field.field_type == "INTEGER" || field.field_type == "FLOAT" {
                field_def.push_str(&format!(" CHECK(\"{}\" <= {})", field.title, max));
            }
        }

        create_table_sql.push_str(&field_def);
    }

    create_table_sql.push_str(", created_at TEXT DEFAULT CURRENT_TIMESTAMP");
    create_table_sql.push_str(", updated_at TEXT DEFAULT CURRENT_TIMESTAMP");
    create_table_sql.push_str(")");

    if let Err(err) = conn.execute(&create_table_sql, []) {
        return Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to create table: {}", err),
        }));
    }

    for field in &data.fields {
        let insert_metadata_sql = "INSERT INTO _database_metadata (table_id, table_name, field_name, field_type, unique_field, nullable, min, max, allowed_extensions) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

        if let Err(err) = conn.execute(
            insert_metadata_sql,
            rusqlite::params![
                table_id,
                data.collection,
                field.title,
                field.field_type,
                field.unique,
                field.nullable,
                field.min,
                field.max,
                field.allowed_extensions
            ],
        ) {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to save field metadata: {}", err),
            }));
        }
    }

    Ok(HttpResponse::Ok().json(Response {
        success: true,
        message: format!("Collection {} has been created!", &data.collection),
    }))
}

fn sql_type_from_field_type(field_type: &str) -> &str {
    match field_type {
        "VARCHAR" => "VARCHAR",
        "TEXT" => "TEXT",
        "INTEGER" => "INTEGER",
        "DECIMAL" => "REAL",
        "BOOLEAN" => "INTEGER",
        "DATETIME" => "TEXT",
        "TIMESTAMP" => "TEXT",
        "FILE" => "TEXT",
        _ => "TEXT",
    }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateCollectionFields {
    title: String,
    #[serde(rename = "type")]
    field_type: String,
    unique: bool,
    nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    allowed_extensions: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateCollectionRequest {
    collection_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    collection_name: Option<String>,
    #[serde(default)]
    fields: Vec<UpdateCollectionFields>,
}

#[post("/update-collection")]
pub async fn update_collection(
    data: web::Json<UpdateCollectionRequest>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, Error> {
    if data.collection_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(Response {
            success: false,
            message: "collection_id is required".to_string(),
        }));
    }

    if let Some(ref name) = data.collection_name {
        if name.starts_with('_') {
            return Ok(HttpResponse::BadRequest().json(Response {
                success: false,
                message: "Collection name cannot start with '_'".to_string(),
            }));
        }
    }

    let conn = match app_data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to get database connection: {}", err),
            }));
        }
    };

    let metadata_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_database_metadata'",
        [],
        |row| row.get(0),
    );

    if let Ok(0) = metadata_exists {
        return Ok(HttpResponse::NotFound().json(Response {
            success: false,
            message: "Metadata table does not exist".to_string(),
        }));
    }

    let current_table_name: Result<String, _> = conn.query_row(
        "SELECT table_name FROM _database_metadata WHERE table_id = ?1 LIMIT 1",
        [&data.collection_id],
        |row| row.get(0),
    );

    let current_table_name = match current_table_name {
        Ok(name) => name,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(HttpResponse::NotFound().json(Response {
                success: false,
                message: format!("Collection with id '{}' not found", data.collection_id),
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to query collection: {}", err),
            }));
        }
    };

    let existing_fields: Vec<(String, String)> = {
        let mut stmt = match conn.prepare(
            "SELECT field_name, field_type FROM _database_metadata WHERE table_name = ?1",
        ) {
            Ok(stmt) => stmt,
            Err(err) => {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to prepare metadata query: {}", err),
                }));
            }
        };

        match stmt.query_map([&current_table_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(err) => {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to fetch existing fields: {}", err),
                }));
            }
        }
    };

    let incoming_field_names: std::collections::HashSet<String> =
        data.fields.iter().map(|f| f.title.clone()).collect();

    let removed_fields: Vec<(String, String)> = existing_fields
        .iter()
        .filter(|(name, _)| !incoming_field_names.contains(name))
        .cloned()
        .collect();

    let removed_file_fields: Vec<String> = removed_fields
        .iter()
        .filter(|(_, field_type)| field_type == "FILE")
        .map(|(name, _)| name.clone())
        .collect();

    if !removed_file_fields.is_empty() {
        let cols = removed_file_fields
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", ");

        let select_query = format!("SELECT {} FROM \"{}\"", cols, current_table_name);

        if let Ok(mut stmt) = conn.prepare(&select_query) {
            let col_count = removed_file_fields.len();
            let _ = stmt
                .query_map([], |row| {
                    let mut paths = Vec::new();
                    for i in 0..col_count {
                        if let Ok(Some(raw)) = row.get::<_, Option<String>>(i) {
                            if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str(&raw) {
                                for entry in arr {
                                    if let Some(p) = entry.as_str() {
                                        paths.push(p.to_string());
                                    }
                                }
                            } else {
                                paths.push(raw);
                            }
                        }
                    }
                    Ok(paths)
                })
                .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
                .map(|all_paths| {
                    for paths in all_paths {
                        for path in paths {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                });
        }
    }

    let existing_field_names: std::collections::HashSet<String> =
        existing_fields.iter().map(|(name, _)| name.clone()).collect();

    let new_fields: Vec<&UpdateCollectionFields> = data
        .fields
        .iter()
        .filter(|f| !existing_field_names.contains(&f.title))
        .collect();

    for field in &new_fields {
        let mut alter_sql = format!(
            "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}",
            current_table_name,
            field.title,
            sql_type_from_field_type(&field.field_type)
        );

        if field.unique {
            alter_sql.push_str(" UNIQUE");
        }

        if let Err(err) = conn.execute(&alter_sql, []) {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to add column '{}': {}", field.title, err),
            }));
        }
    }

    for (field_name, _) in &removed_fields {
        if let Err(err) = conn.execute(
            &format!(
                "ALTER TABLE \"{}\" DROP COLUMN \"{}\"",
                current_table_name, field_name
            ),
            [],
        ) {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to drop column '{}': {}", field_name, err),
            }));
        }
    }

    let target_table_name = match &data.collection_name {
        Some(new_name) if new_name != &current_table_name => {
            let name_taken: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [new_name],
                |row| row.get(0),
            );

            if let Ok(count) = name_taken {
                if count > 0 {
                    return Ok(HttpResponse::BadRequest().json(Response {
                        success: false,
                        message: format!("A collection named '{}' already exists", new_name),
                    }));
                }
            }

            if let Err(err) = conn.execute(
                &format!(
                    "ALTER TABLE \"{}\" RENAME TO \"{}\"",
                    current_table_name, new_name
                ),
                [],
            ) {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to rename table: {}", err),
                }));
            }

            if let Err(err) = conn.execute(
                "UPDATE _database_metadata SET table_name = ?1 WHERE table_id = ?2",
                rusqlite::params![new_name, data.collection_id],
            ) {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to update metadata table name: {}", err),
                }));
            }

            new_name.clone()
        }
        _ => current_table_name.clone(),
    };

    for (field_name, _) in &removed_fields {
        if let Err(err) = conn.execute(
            "DELETE FROM _database_metadata WHERE table_name = ?1 AND field_name = ?2",
            rusqlite::params![target_table_name, field_name],
        ) {
            return Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to remove metadata for field '{}': {}", field_name, err),
            }));
        }
    }

    for field in &data.fields {
        if existing_field_names.contains(&field.title) {
            if let Err(err) = conn.execute(
                "UPDATE _database_metadata SET field_type = ?1, unique_field = ?2, nullable = ?3, min = ?4, max = ?5, allowed_extensions = ?6, updated_at = CURRENT_TIMESTAMP WHERE table_name = ?7 AND field_name = ?8",
                rusqlite::params![
                    field.field_type,
                    field.unique,
                    field.nullable,
                    field.min,
                    field.max,
                    field.allowed_extensions,
                    target_table_name,
                    field.title,
                ],
            ) {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to update metadata for field '{}': {}", field.title, err),
                }));
            }
        } else {
            if let Err(err) = conn.execute(
                "INSERT INTO _database_metadata (table_id, table_name, field_name, field_type, unique_field, nullable, min, max, allowed_extensions) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    data.collection_id,
                    target_table_name,
                    field.title,
                    field.field_type,
                    field.unique,
                    field.nullable,
                    field.min,
                    field.max,
                    field.allowed_extensions,
                ],
            ) {
                return Ok(HttpResponse::InternalServerError().json(Response {
                    success: false,
                    message: format!("Failed to insert metadata for field '{}': {}", field.title, err),
                }));
            }
        }
    }

    Ok(HttpResponse::Ok().json(Response {
        success: true,
        message: format!("Collection '{}' updated successfully", target_table_name),
    }))
}