use std::ops::AddAssign;

use rocket::form::validate::Contains;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{Parameter, ParameterValue};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use user_agent_parser::{Device, Engine, OS};

use crate::database::auth_token::AuthToken;
use crate::database::database::DBPool;
use crate::database::schema::*;
use crate::database::user::User;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ErrorResponder;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_id = request.headers().get_one("X-User-Id").map(|s| s.parse::<u32>().ok()).flatten();
        let auth_token = request.headers().get_one("X-Auth-Token").map(|s| hex::decode(s).ok()).flatten();
        if user_id.is_none() || auth_token.is_none() {
            return Outcome::Error((Status::Unauthorized, ErrorType::UserNotFound.res()));
        }

        let db: &DBPool = request.rocket().state::<DBPool>().unwrap();
        let conn = &mut db.get().unwrap();

        let result = User::find_logged_in_opt(conn, user_id.unwrap(), auth_token.unwrap());

        if let Some((user, auth)) = result.ok().flatten() {
            if user.status == UserStatus::Unconfirmed {
                return Outcome::Error((Status::Unauthorized, ErrorType::UserUnconfirmed.res()));
            }
            if user.status == UserStatus::Banned {
                return Outcome::Error((Status::Unauthorized, ErrorType::UserBanned.res()));
            }

            let result = auth.update_last_use_date(conn);
            if result.is_err() {
                // TODO: log the error but keep the response as successful
            }
            return Outcome::Success(user);
        }
        Outcome::Error((Status::Unauthorized, ErrorType::UserNotFound.res()))
    }
}
impl OpenApiFromRequest<'_> for User {
    fn from_request_input(gen: &mut OpenApiGenerator, name: String, required: bool) -> rocket_okapi::Result<RequestHeaderInput> {
        // Specify needed header: X-User-Id and X-Auth-Token
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "X-User-Id".to_string(),
            location: "".to_string(),
            description: None,
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema: gen.json_schema::<u32>(),
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        }))
    }
}
pub struct UserAuthInfo {
    pub user_id: Option<u32>,
    pub auth_token: Option<Vec<u8>>,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuthInfo {
    type Error = ErrorResponder;
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_id = User::get_id_from_headers(request);
        let auth_token = AuthToken::get_auth_token_from_headers(request);
        Outcome::Success(UserAuthInfo {
            user_id,
            auth_token,
        })
    }
}
impl OpenApiFromRequest<'_> for UserAuthInfo {
    fn from_request_input(gen: &mut OpenApiGenerator, name: String, required: bool) -> rocket_okapi::Result<RequestHeaderInput> {
        // Specify needed header: X-User-Id and X-Auth-Token
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "X-User-Id".to_string(),
            location: "".to_string(),
            description: None,
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema: gen.json_schema::<u32>(),
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        }))
    }
}


#[derive(Debug)]
pub struct DeviceInfo {
    pub(crate) device_string: String,
    pub(crate) ip_address: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DeviceInfo {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut ip_address = request.remote().map(|s| s.to_string()).or(request.headers().get_one("X-Forwarded-For").map(|s| s.to_string()));

        let device = Device::from_request(request).await.unwrap();
        let os = OS::from_request(request).await.unwrap();
        let engine = Engine::from_request(request).await.unwrap();

        let device_string = device_str(device, os, engine);

        // removing port from ip address even if it is an ipv6
        if let Some(ip) = ip_address.clone() {
            if ip.contains(':') {
                if ip.chars().filter(|c| *c == 'z').count() > 1 {
                    if ip.starts_with('[') && ip.contains("]") {
                        ip_address = Some(ip[1..ip.find("]").unwrap()].to_string());
                    }
                } else {
                    ip_address = Some(ip[0..ip.find(":").unwrap()].to_string());
                }
            }
        }

        Outcome::Success(DeviceInfo {
            device_string,
            ip_address,
        })
    }
}
impl OpenApiFromRequest<'_> for DeviceInfo {
    fn from_request_input(gen: &mut OpenApiGenerator, name: String, required: bool) -> rocket_okapi::Result<RequestHeaderInput> {
        // Specify needed header: user-agent
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "User-Agent".to_string(),
            location: "".to_string(),
            description: None,
            required: false,
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Schema {
                style: None,
                explode: None,
                allow_reserved: false,
                schema: gen.json_schema::<String>(),
                example: None,
                examples: None,
            },
            extensions: Default::default(),
        }))
    }
}

fn device_str(device: Device, os: OS, engine: Engine) -> String {
    let mut device_str = String::new();

    if let Some(brand) = device.brand {
        device_str = format!("{} ", brand);
    }
    if let Some(name) = device.name {
        device_str.add_assign(format!("{} ", name).as_str());
    } else if let Some(model) = device.model {
        device_str.add_assign(format!("{} ", model).as_str());
    }

    if let Some(name) = os.name {
        device_str.add_assign(format!("({}", name).as_str());
        if let Some(major) = os.major {
            device_str.add_assign(format!(" {}", major).as_str());
            if let Some(minor) = os.minor {
                device_str.add_assign(format!(".{}", minor).as_str());
                if let Some(patch) = os.patch {
                    device_str.add_assign(format!(".{}", patch).as_str());
                    if let Some(patch_minor) = os.patch_minor {
                        device_str.add_assign(format!(".{}", patch_minor).as_str());
                    }
                }
            }
        }
        device_str.add_assign(") ");
    }

    if let Some(name) = engine.name {
        device_str.add_assign(format!("{}", name).as_str());
        if let Some(major) = engine.major {
            device_str.add_assign(format!(" {}", major).as_str());
            if let Some(minor) = engine.minor {
                device_str.add_assign(format!(".{}", minor).as_str());
                if let Some(patch) = engine.patch {
                    device_str.add_assign(format!(".{}", patch).as_str());
                }
            }
        }
    }

    if device_str.is_empty() {
        device_str = "Unknown".to_string();
    }
    device_str
}
