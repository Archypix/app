use crate::database::auth_token::{AuthToken, Confirmation};
use crate::database::database::{DBConn, DBPool};
use crate::database::schema::ConfirmationAction;
use crate::database::schema::UserStatus;
use crate::database::user::User;
use crate::utils::auth::{DeviceInfo, UserAuthInfo};
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use crate::utils::validation::validate_input;
use diesel::Connection;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct ConfirmCodeData {
    action: ConfirmationAction,
    code_token: String,
    #[validate(range(min = 0, max = 9999, message = "Code must be a 4 digit number"))]
    code: u16,
}

#[derive(Deserialize, Debug, Validate)]
pub struct ConfirmTokenData {
    action: ConfirmationAction,
    token: String,
    user_id: u32,
}

#[derive(Serialize, Debug)]
pub struct ConfirmResponse {
    pub(crate) auth_token: Option<String>,
}

#[post("/auth/confirm/code", data = "<data>")]
pub fn auth_confirm_code(data: Json<ConfirmCodeData>, db: &rocket::State<DBPool>, user_auth_info: UserAuthInfo, device_info: DeviceInfo) -> Result<Json<ConfirmResponse>, ErrorResponder> {
    validate_input(&data)?;
    let conn: &mut DBConn = &mut db.get().unwrap();
    let user_id = user_auth_info.user_id.ok_or(ErrorType::UserNotFound.to_responder())?;
    let user = User::from_id(conn, &user_id)?;

    let code_token = hex::decode(&data.code_token).map_err(|_| ErrorType::UnprocessableEntity.to_responder())?;

    match data.action {
        ConfirmationAction::Signup => {
            conn.transaction::<_, ErrorResponder, _>(|conn| {
                // It is useless to check if user status is Unconfirmed. Only one signup confirm can exist at a time.
                Confirmation::check_code_and_mark_as_used(conn, &user_id, &data.action, &code_token, &data.code)?;
                user.switch_status(conn, &UserStatus::Normal)?;

                let auth_token = AuthToken::insert_token_for_user(conn, &user.id, &device_info, 0)?;

                Ok(Json(ConfirmResponse {
                    auth_token: Some(hex::encode(auth_token)),
                }))
            })
        }
        _ => {
            ErrorType::BadRequest.to_err()
        }
    }
}
