use pwhash::bcrypt;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

use crate::database::auth_token::AuthToken;
use crate::database::database::{DBConn, DBPool};
use crate::database::schema::UserStatus;
use crate::database::user::User;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};

#[derive(Deserialize, Debug)]
pub struct SigninData {
    email: String,
    password: String,
}

#[derive(Serialize, Debug)]
pub struct SigninResponse {
    pub(crate) user_id: u32,
    pub(crate) auth_token: String,
}

#[post("/auth/signin", data = "<data>")]
pub fn auth_signin(data: Json<SigninData>, db: &rocket::State<DBPool>, device_info: DeviceInfo) -> Result<Json<SigninResponse>, ErrorResponder> {
    let conn: &mut DBConn = &mut db.get().unwrap();

    let user = User::find_by_email_opt(conn, &data.email)
        .and_then(|user| {
            if let Some(user) = user {
                if bcrypt::verify(data.password.clone(), &*user.password_hash) {
                    return Ok(user);
                }
            }
            ErrorType::InvalidEmailOrPassword.to_err()
        })?;

    return match user.status {
        UserStatus::Banned => {
            ErrorType::UserBanned.to_err()
        }
        UserStatus::Unconfirmed => {
            ErrorType::UserUnconfirmed.to_err()
        }
        _ => {
            let auth_token = AuthToken::insert_token_for_user(conn, &user.id, &device_info, 0)?;

            Ok(Json(SigninResponse {
                user_id: user.id,
                auth_token: hex::encode(auth_token),
            }))
        }
    };
}
