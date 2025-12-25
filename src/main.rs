
use actix_web::{get, post, web, App, HttpServer, HttpResponse, HttpRequest, Result, Responder};
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
    serde: f32,
    serde_json: f32,
    rust_embed: f32,
    mime_guess: f32
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
    println!("🚀 Listening at http://127.0.0.1:{}", port);
    HttpServer::new(|| {
        App::new().service(index).service(web::scope("/api").service(get_version)).default_service(web::route().to(static_files))
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
        rusqlite: 0.38,
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
        rusqlite: 0.38,
        serde: 1.0,
        serde_json: 1.0,
        rust_embed: 8.0,
        mime_guess: 2.0
    }))
}
