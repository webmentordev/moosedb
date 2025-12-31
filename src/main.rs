
use actix_web::{get, post, web, App, HttpServer, HttpResponse, HttpRequest, HttpMessage, Result, Responder, middleware};
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
    /// Coming soon
    Update,
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

type DbPool = Pool<SqliteConnectionManager>;
struct AppData {
    database: DbPool,
    jwt_secret: String,
    configs: HashMap<String, String>
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
        Some(Commands::Update) => {
            println!("coming soon...");
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

            let configs = moosedb::load_configs(&conn).unwrap();
            let jwt_secret = configs.get("secret").unwrap().clone();
            
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
