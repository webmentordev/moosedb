use crate::AppData;
use crate::Claims;
use actix_web::dev::ServiceRequest;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, Result, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct LoginRequest {
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

pub async fn login(
    data: web::Data<AppData>,
    credentials: web::Json<LoginRequest>,
) -> impl Responder {
    let conn = match data.database.get() {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                message: "Database connection failed.".to_string(),
            });
        }
    };

    let result: Result<(String, String), rusqlite::Error> = conn.query_row(
        "SELECT email, password from _super_admins WHERE email = ?1",
        [&credentials.email],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

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
            } else {
                HttpResponse::Unauthorized().json(ErrorResponse {
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

pub async fn validator(
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

fn create_jwt(
    email: &str,
    user_id: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
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

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[derive(Deserialize)]
struct UpdatePasswordRequest {
    new_password: String,
    confirm_new_password: String,
}

#[post("/update-your-password")]
pub async fn update_your_password(
    req: HttpRequest,
    body: web::Json<UpdatePasswordRequest>,
    data: web::Data<AppData>,
) -> impl Responder {
    if body.new_password.trim().is_empty() || body.confirm_new_password.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Password fields cannot be empty"
        }));
    }

    if body.new_password != body.confirm_new_password {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Passwords do not match"
        }));
    }

    let email = match req.extensions().get::<Claims>().map(|c| c.email.clone()) {
        Some(e) => e,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            }));
        }
    };

    let pool = data.database.clone();
    let new_password = body.new_password.clone();

    let result = web::block(move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let conn = pool.get()?;

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM _super_admins WHERE email = ?1)",
            params![email],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(false);
        }

        let hashed = hash(new_password, DEFAULT_COST)?;

        conn.execute(
            "UPDATE _super_admins SET password = ?1, updated_at = CURRENT_TIMESTAMP WHERE email = ?2",
            params![hashed, email],
        )?;

        Ok(true)
    })
    .await;

    match result {
        Ok(Ok(true)) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Password updated successfully"
        })),
        Ok(Ok(false)) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": "User not found"
        })),
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "Failed to update password"
        })),
    }
}
