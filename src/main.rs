
use actix_web::{get, post, web, App, HttpServer, HttpResponse, HttpRequest, HttpMessage, Result, Responder, middleware, Error};
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::path::Path;
use r2d2::Pool;
use serde::{Serialize, Deserialize};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::time::{SystemTime, UNIX_EPOCH};
use bcrypt::verify;
use clap::{Parser, Subcommand};
use std::sync::{RwLock, Arc};
use moosedb::random_numbers;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Asset;



/// MooseDB CLI
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the MooseDB server
    Serve {
        /// Option to change the host (Optional)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Option to change the port (Optional)
        #[arg(long, default_value_t = 8855)]
        port: u16,
    },
    /// Update super admin credientials
    Upsuper{
        /// Email of the super admin (Required)
        #[arg(long = "email", short = 'e', required = true, value_name = "EMAIL")]
        email: String,

        /// New password of the super admin (Required)
        #[arg(long = "password", short = 'p', required = true, value_name = "PASSWORD")]
        password: String,
    },
}


#[derive(Serialize)]
struct Info {
    version: f32,
    actix_web: f32,
    actix_files: f32,
    rusqlite: f32,
    r2d2: f32,
    r2d2_sqlite: f32,
    serde: f32,
    serde_json: f32,
    rust_embed: f32,
    mime_guess: f32
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    email: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}


#[derive(Serialize)]
struct LoginResponse {
    token: String,
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}


#[derive(Deserialize)]
struct GetSetting {
    key: String
}

#[derive(Serialize)]
struct SendSetting {
    success: bool,
    value: String
}

#[derive(Deserialize)]
struct UpdateSetting {
    key: String,
    value: String
}

#[derive(Serialize)]
struct Response {
    success: bool,
    message: String
}


#[derive(Deserialize)]
struct CollectionID {
    collection_id: String
}

#[derive(Serialize, Deserialize)]
struct CollectionRecords {
    success: bool,
    message: String,
    records: Option<Vec<serde_json::Value>>
}


type DbPool = Pool<SqliteConnectionManager>;
struct AppData {
    database: DbPool,
    jwt_secret: String,
    configs: Arc<RwLock<HashMap<String, String>>>,
}

async fn static_files(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');
    let file_path = if path.is_empty() { "index.html" } else { path };

    match Asset::get(file_path) {
        Some(content) => {
            let body = content.data.into_owned();
            let mime = from_path(file_path).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(body)
        }
        None => match Asset::get("index.html") {
            Some(index_file) => HttpResponse::Ok()
                .content_type("text/html")
                .body(index_file.data.into_owned()),
            None => HttpResponse::NotFound().body("404 Not Found"),
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.command {
        None => {
            Args::parse_from(&["moosedb", "--help"]);
            Ok(())
        }
        Some(Commands::Upsuper { email, password}) => {
            match moosedb::update_super_user(email, password) {
                Ok(_) => println!("Super admin's password has been updated!"),
                Err(_) => println!("Password update failed!")
            }
            Ok(())
        }
        Some(Commands::Serve { host, port }) => {
            let mut create_new_db = false;
            let file_exists = Path::new("database.sqlite").exists();
            
            if !file_exists {
                create_new_db = true;
            }
            let manager = SqliteConnectionManager::file("database.sqlite");
            let pool = Pool::new(manager).expect("Failed to create pool");
            let conn = pool.get().expect("Failed to get connection");
            if let Err(e) = moosedb::initialize_db(&conn, create_new_db) {
                println!("Database could not be created: {}", e);
                return Ok(());
            }

            let configs = Arc::new(RwLock::new(moosedb::load_configs(&conn).unwrap()));
            let jwt_secret = configs.read().unwrap().get("secret").unwrap().clone();
            
            println!("🚀 Listening at http://{}:{}", host, port);
            
            HttpServer::new(move || {
                let auth = HttpAuthentication::bearer(validator);
                App::new()
                    .app_data(web::Data::new(AppData { 
                        database: pool.clone(),
                        jwt_secret: jwt_secret.clone(),
                        configs: configs.clone()
                    }))
                    .wrap(middleware::Logger::default())
                    .service(index)
                    .route("/auth/login", web::post().to(login))
                    .service(
                        web::scope("/admin/api")
                            .wrap(auth.clone())
                            .service(get_version)
                            .service(get_setting)
                            .service(update_setting)
                            .service(create_collection)
                            .service(get_collections)
                            .service(delete_collection)
                            .service(get_collection_records)
                    )
                    .service(web::scope("/api").service(get_version))
                    .default_service(web::route().to(static_files))
            })
            .bind((host, port))?
            .run()
            .await
        }
    }
}


#[get("/")]
async fn index() -> Result<impl Responder> {
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
        mime_guess: 2.0
    }))
}


#[post("/get-version")]
async fn get_version() -> Result<impl Responder> {
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
        mime_guess: 2.0
    }))
}

#[post("/get-setting")]
async fn get_setting(
    data: web::Data<AppData>,
    request: web::Json<GetSetting>
) -> Result<impl Responder> {
    let configs = data.configs.read().unwrap();
    match configs.get(&request.key) {
        Some(value) => {
            Ok(web::Json(SendSetting {
                success: true,
                value: value.clone()
            }))
        }
        None => {
            Ok(web::Json(SendSetting {
                success: false,
                value: String::new()
            }))
        }
    }
}

#[post("/update-setting")]
async fn update_setting(
    data: web::Data<AppData>,
    request: web::Json<UpdateSetting>
) -> Result<impl Responder> {
    match moosedb::update_setting(request.key.to_string(), request.value.to_string()) {
        Ok(_) => {
            let mut configs = data.configs.write().unwrap();
            configs.insert(request.key.to_string(), request.value.to_string());
            
            Ok(web::Json(Response {
                success: true,
                message: "Setting updated successfully".to_string()
            }))
        }
        Err(err) => {
            Ok(web::Json(Response {
                success: false,
                message: err.to_string()
            }))
        }
    }
}


#[post("/get-collection-records")]
async fn get_collection_records(
    data: web::Data<AppData>,
    request: web::Json<CollectionID>
) -> Result<impl Responder> {
    let collection_id = request.collection_id.clone();
    
    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
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
        return Ok(HttpResponse::NotFound().json(CollectionRecords {
            success: false,
            message: "Metadata table does not exist".to_string(),
            records: None,
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
            }));
        }
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to query collection: {}", err),
                records: None,
            }));
        }
    };

    let mut stmt = match conn.prepare(&format!("SELECT * FROM \"{}\"", table_name)) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
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
            return Ok(HttpResponse::InternalServerError().json(CollectionRecords {
                success: false,
                message: format!("Failed to query records: {}", err),
                records: None,
            }));
        }
    };

    let records: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();

    Ok(HttpResponse::Ok().json(CollectionRecords {
        success: true,
        message: format!("Retrieved {} records from '{}'", records.len(), table_name),
        records: Some(records),
    }))
}



#[post("/delete-collection")]
async fn delete_collection(
    data: web::Data<AppData>,
    request: web::Json<CollectionID>
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
async fn get_collections(
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, Error> {
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
    let mut stmt = match conn.prepare("SELECT DISTINCT table_id, table_name FROM _database_metadata") {
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
        Ok(collections) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "collections": collections
            })))
        }
        Err(err) => {
            Ok(HttpResponse::InternalServerError().json(Response {
                success: false,
                message: format!("Failed to fetch collections: {}", err),
            }))
        }
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
}

#[post("/create-collection")]
async fn create_collection(
    data: web::Json<CollectionRequest>,
    app_data: web::Data<AppData>
) -> Result<HttpResponse, Error> {
    if data.collection.is_empty() {
        return Ok(HttpResponse::BadRequest().json(Response {
            success: false,
            message: "Collection name is required".to_string(),
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
        "CREATE TABLE \"{}\" (id INTEGER PRIMARY KEY AUTOINCREMENT",
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
                field_def.push_str(&format!(" CHECK(\"{}\") >= {})", field.title, min));
            }
        }

        if let Some(max) = field.max {
            if field.field_type == "VARCHAR" || field.field_type == "TEXT" {
                field_def.push_str(&format!(" CHECK(length(\"{}\") <= {})", field.title, max));
            } else if field.field_type == "INTEGER" || field.field_type == "FLOAT" {
                field_def.push_str(&format!(" CHECK(\"{}\") <= {})", field.title, max));
            }
        }

        create_table_sql.push_str(&field_def);
    }

    create_table_sql.push_str(")");

    if let Err(err) = conn.execute(&create_table_sql, []) {
        return Ok(HttpResponse::InternalServerError().json(Response {
            success: false,
            message: format!("Failed to create table: {}", err),
        }));
    }

    for field in &data.fields {
        let insert_metadata_sql = "INSERT INTO _database_metadata (table_id, table_name, field_name, field_type, unique_field, nullable, min, max) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

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
                field.max
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
        _ => "TEXT",
    }
}

async fn login(
    data: web::Data<AppData>,
    credentials: web::Json<LoginRequest>,
) -> impl Responder {

    let conn = match data.database.get(){
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse{
                success: false,
                message: "Database connection failed.".to_string()
            });
        }
    };

    let result: Result<(String, String), rusqlite::Error> = conn.query_row("SELECT email, password from _super_admins WHERE email = ?1", 
    [&credentials.email], 
    |row| Ok((row.get(0)?, row.get(1)?))); 

    match result {
        Ok((email, hashed_password)) => {
            let is_valid = verify(&credentials.password, &hashed_password).unwrap_or(false);
            if is_valid {
                match create_jwt(&email, &email, &data.jwt_secret) {
                    Ok(token) => HttpResponse::Ok().json(LoginResponse {
                        token,
                        success: true,
                        message: "Login successful".to_string(),
                    }),
                    Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                        success: false,
                        message: "Failed to create token".to_string(),
                    }),
                }
            }else{
                HttpResponse::Unauthorized().json(ErrorResponse{
                    success: false,
                    message: "Email or Password does not match.".to_string(),
                })
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(ErrorResponse {
            success: false,
            message: "Email not found!".to_string(),
        }),
    }
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let jwt_secret = req
        .app_data::<web::Data<AppData>>()
        .map(|data| data.jwt_secret.clone())
        .unwrap_or_default();

    match verify_jwt(credentials.token(), &jwt_secret) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            let error = actix_web::error::ErrorUnauthorized("Invalid token");
            Err((error, req))
        }
    }
}

fn create_jwt(email: &str, user_id: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: now + 3600,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
