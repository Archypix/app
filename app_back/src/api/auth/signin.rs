use pwhash::bcrypt;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use std::env;

use crate::database::auth_token::{AuthToken, Confirmation, TOTPSecret};
use crate::database::database::{DBConn, DBPool};
use crate::database::schema::{ConfirmationAction, UserStatus};
use crate::database::user::User;
use crate::mailing::mailer::send_rendered_email;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use crate::utils::utils::left_pad;

#[derive(Deserialize, Debug)]
pub struct SigninData {
    email: String,
    password: String,
    totp_code: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct SigninResponse {
    pub(crate) user_id: u32,
    pub(crate) auth_token: String,
    pub(crate) name: String,
    pub(crate) status: UserStatus,
}

#[derive(Serialize, Debug)]
pub struct SigninEmailResponse {
    pub(crate) code_token: String,
}

#[post("/auth/signin", data = "<data>")]
pub fn auth_signin(data: Json<SigninData>, db: &rocket::State<DBPool>, device_info: DeviceInfo) -> Result<Json<SigninResponse>, ErrorResponder> {
    let conn: &mut DBConn = &mut db.get().unwrap();
    let user = check_user_password_and_status(conn, &data.email, &data.password)?;

    if user.tfa_login {
        if let Some(totp_code) = &data.totp_code {
            if !TOTPSecret::check_user_totp(conn, &user.id, totp_code)? {
                return ErrorType::InvalidTOTPCode.to_err();
            }
        } else {
            // 2FA Required, checking if TOTP is available
            if TOTPSecret::has_user_totp(conn, &user.id) {
                return ErrorType::TFARequired.to_err();
            }
            return ErrorType::TFARequiredOverEmail.to_err();
        }
    }

    let auth_token = AuthToken::insert_token_for_user(conn, &user.id, &device_info, 0)?;

    Ok(Json(SigninResponse {
        user_id: user.id,
        auth_token: hex::encode(auth_token),
        name: user.name,
        status: user.status,
    }))
}

#[post("/auth/signin/email", data = "<data>")]
pub fn auth_signin_email(data: Json<SigninData>, db: &rocket::State<DBPool>, device_info: DeviceInfo) -> Result<Json<SigninEmailResponse>, ErrorResponder> {
    let conn: &mut DBConn = &mut db.get().unwrap();
    let user = check_user_password_and_status(conn, &data.email, &data.password)?;

    let (token, code_token, code) = Confirmation::insert_confirmation(conn, user.id, ConfirmationAction::Signin, &device_info, 0)?;
    let code_str = left_pad(&code.to_string(), '0', 4);

    // Sending email
    let signin_url = format!("{}/signin/confirm?id={}&token={}",
                             env::var("FRONTEND_HOST").expect("FRONTEND_HOST must be set"), user.id, hex::encode(&token));
    let subject = "Confirm your email address".to_string();
    let mut context = tera::Context::new();
    context.insert("name", &user.name);
    context.insert("url", &signin_url);
    context.insert("code", &code_str);
    context.insert("ip", &device_info.ip_address.unwrap_or("Unknown".to_string()));
    context.insert("agent", &device_info.device_string);
    send_rendered_email((user.name.clone(), data.email.clone()), subject, "confirm_signin".to_string(), context);

    Ok(Json(SigninEmailResponse {
        code_token: hex::encode(code_token),
    }))
}


fn check_user_password_and_status(conn: &mut DBConn, email: &str, password: &str) -> Result<User, ErrorResponder> {
    let user = User::find_by_email_opt(conn, email)
        .and_then(|user| {
            if let Some(user) = user {
                if bcrypt::verify(password, &*user.password_hash) {
                    return Ok(user);
                }
            }
            ErrorType::InvalidEmailOrPassword.to_err()
        })?;

    match user.status {
        UserStatus::Banned => {
            ErrorType::UserBanned.to_err()
        }
        UserStatus::Unconfirmed => {
            ErrorType::UserUnconfirmed.to_err()
        }
        _ => {
            Ok(user)
        }
    }
}
