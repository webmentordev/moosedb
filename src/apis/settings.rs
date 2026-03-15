use crate::AppData;
use crate::db::connection::update_setting;

use actix_web::{Responder, Result, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GetSetting {
    key: String,
}

#[derive(Serialize)]
struct SendSetting {
    success: bool,
    value: String,
    message: String,
}

#[derive(Deserialize)]
struct UpdateSetting {
    key: String,
    value: String,
}

#[derive(Serialize)]
pub struct Response {
    pub success: bool,
    pub message: String,
}

#[post("/get-setting")]
async fn get_setting(
    data: web::Data<AppData>,
    request: web::Json<GetSetting>,
) -> Result<impl Responder> {
    let configs = data.configs.read().unwrap();
    if &request.key != "secret" {
        match configs.get(&request.key) {
            Some(value) => Ok(web::Json(SendSetting {
                success: true,
                value: value.clone(),
                message: "Value has been found".to_string(),
            })),
            None => Ok(web::Json(SendSetting {
                success: false,
                value: String::new(),
                message: "Value does not exist! creating new value.".to_string(),
            })),
        }
    } else {
        Ok(web::Json(SendSetting {
            success: false,
            value: String::new(),
            message: "Action is not allowed!".to_string(),
        }))
    }
}

#[post("/update-setting")]
async fn update_setting_func(
    data: web::Data<AppData>,
    request: web::Json<UpdateSetting>,
) -> Result<impl Responder> {
    let key = request.key.to_string();
    if key != "secret".to_string() {
        match update_setting(key.clone(), request.value.to_string()) {
            Ok(_) => {
                let mut configs = data.configs.write().unwrap();
                configs.insert(key.clone(), request.value.to_string());

                Ok(web::Json(Response {
                    success: true,
                    message: "Setting updated successfully".to_string(),
                }))
            }
            Err(err) => Ok(web::Json(Response {
                success: false,
                message: err.to_string(),
            })),
        }
    } else {
        Ok(web::Json(Response {
            success: false,
            message: "Action is not allowed".to_string(),
        }))
    }
}
