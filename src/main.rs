mod apis;
mod db;
mod utils;

use apis::auth::*;
use apis::collections::*;
use apis::public::*;
use apis::records::*;
use apis::settings::*;
use db::connection::*;

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result, get, middleware, web,
};

use actix_web_httpauth::middleware::HttpAuthentication;
use clap::{Parser, Subcommand};
use env_logger::Builder;
use mime_guess::from_path;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Arc, RwLock};

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
    Upsuper {
        /// Email of the super admin (Required)
        #[arg(long = "email", short = 'e', required = true, value_name = "EMAIL")]
        email: String,

        /// New password of the super admin (Required)
        #[arg(
            long = "password",
            short = 'p',
            required = true,
            value_name = "PASSWORD"
        )]
        password: String,
    },
    /// Update the system’s secret token.
    Upsecret,
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
    mime_guess: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    email: String,
}

type DbPool = Pool<SqliteConnectionManager>;
struct AppData {
    database: DbPool,
    jwt_secret: String,
    configs: Arc<RwLock<HashMap<String, String>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.command {
        None => {
            Args::parse_from(&["moosedb", "--help"]);
            Ok(())
        }
        Some(Commands::Upsecret) => {
            match update_secret_key() {
                Ok(_) => println!("Secret token has been updated!"),
                Err(error) => println!("Secret update failed! Reason: {}", error),
            }
            Ok(())
        }
        Some(Commands::Upsuper { email, password }) => {
            match update_super_user(email, password) {
                Ok(_) => println!("Super admin's password has been updated!"),
                Err(_) => println!("Password update failed!"),
            }
            Ok(())
        }
        Some(Commands::Serve { host, port }) => {
            let mut create_new_db = false;
            let file_exists = Path::new("database.sqlite").exists();

            Builder::from_env(env_logger::Env::new().default_filter_or("error")).init();

            if !file_exists {
                create_new_db = true;
            }
            let manager = SqliteConnectionManager::file("database.sqlite");
            let pool = Pool::new(manager).expect("Failed to create pool");
            let conn = pool.get().expect("Failed to get connection");
            if let Err(e) = initialize_db(&conn, create_new_db) {
                println!("Database could not be created: {}", e);
                return Ok(());
            }

            let configs = Arc::new(RwLock::new(load_configs(&conn).unwrap()));
            let jwt_secret = configs.read().unwrap().get("secret").unwrap().clone();

            println!("🚀 Listening at http://{}:{}", host, port);

            HttpServer::new(move || {
                let auth = HttpAuthentication::bearer(validator);
                App::new()
                    .app_data(web::Data::new(AppData {
                        database: pool.clone(),
                        jwt_secret: jwt_secret.clone(),
                        configs: configs.clone(),
                    }))
                    .app_data(web::JsonConfig::default().limit(50 * 1024 * 1024))
                    .wrap(middleware::Logger::default())
                    .service(index)
                    .route("/auth/login", web::post().to(login))
                    .route("/uploads/{filename}", web::get().to(serve_upload))
                    .service(
                        web::scope("/admin/api")
                            .wrap(auth.clone())
                            .service(get_version)
                            .service(get_setting)
                            .service(update_setting_func)
                            .service(create_collection)
                            .service(get_collections)
                            .service(delete_collection)
                            .service(get_collection_records)
                            .service(create_super_admin_func)
                            .service(get_super_admins)
                            .service(create_record)
                            .service(update_your_password)
                            .service(delete_collection_records),
                    )
                    .service(
                        web::scope("/api")
                            .service(get_version)
                            .service(get_collection_data)
                            .service(get_single_record),
                    )
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
        mime_guess: 2.0,
    }))
}

async fn serve_upload(req: HttpRequest) -> HttpResponse {
    let filename = req.match_info().get("filename").unwrap_or("");
    let file_path = Path::new("uploads").join(filename);

    match std::fs::read(&file_path) {
        Ok(bytes) => {
            let mime = from_path(&file_path).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(bytes)
        }
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
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
