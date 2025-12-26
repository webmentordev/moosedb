
use actix_web::{get, post, web, App, HttpServer, HttpResponse, HttpRequest, Result, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use serde::{Serialize};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Asset;

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

type DbPool = Pool<SqliteConnectionManager>;
struct AppData {
    database: DbPool,
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
    let port = 8855;
    
    let manager = SqliteConnectionManager::file("database.sqlite");
    let pool = Pool::new(manager).expect("Failed to create pool");
    {
        let conn = pool.get().expect("Failed to get connection");
        if let Err(e) = moosedb::initialize_db(&conn) {
            println!("Database could not be created: {}", e);
            return Ok(());
        }
    }
    
    println!("🚀 Listening at http://127.0.0.1:{}", port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData { 
                database: pool.clone() 
            }))
            .service(index)
            .service(web::scope("/api").service(get_version))
            .default_service(web::route().to(static_files))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
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
