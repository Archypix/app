use crate::api::auth::signin::SigninResponse;
use crate::database::auth_token::{AuthToken, Confirmation};
use crate::database::database::{DBConn, DBPool};
use crate::database::schema::ConfirmationAction;
use crate::database::schema::UserStatus;
use crate::database::user::User;
use crate::utils::auth::{DeviceInfo, UserAuthInfo};
use crate::utils::errors_catcher::{err_transaction, ErrorResponder, ErrorType};
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
pub struct ConfirmSignUpResponse {
    pub(crate) auth_token: String,
}

pub type ConfirmSignInResponse = SigninResponse;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ConfirmResponse {
    SignUp(ConfirmSignUpResponse),
    SignIn(ConfirmSignInResponse),
}

#[post("/auth/confirm/code", data = "<data>")]
pub fn auth_confirm_code(data: Json<ConfirmCodeData>, db: &rocket::State<DBPool>, user: Option<User>, user_auth_info: UserAuthInfo, device_info: DeviceInfo) -> Result<Json<ConfirmResponse>, ErrorResponder> {
    validate_input(&data)?;
    let conn: &mut DBConn = &mut db.get().unwrap();
    let user_id = user_auth_info.user_id.ok_or(ErrorType::UserNotFound.res())?;
    let user = User::from_id(conn, &user_id)?;

    let code_token = hex::decode(&data.code_token).map_err(|_| ErrorType::UnprocessableEntity.res())?;

    err_transaction(conn, |conn| {
        Confirmation::check_code_and_mark_as_used(conn, &user_id, &data.action, &code_token, &data.code, 15)?;

        match data.action {
            ConfirmationAction::Signup => {
                // It is useless to check if user status is Unconfirmed. Only one signup confirm can exist at a time.
                user.switch_status(conn, &UserStatus::Normal)?;

                let auth_token = AuthToken::insert_token_for_user(conn, &user.id, &device_info, 0)?;

                Ok(Json(ConfirmResponse::SignUp(ConfirmSignUpResponse {
                    auth_token: hex::encode(auth_token),
                })))
            }
            ConfirmationAction::Signin => {
                let auth_token = AuthToken::insert_token_for_user(conn, &user.id, &device_info, 0)?;

                Ok(Json(ConfirmResponse::SignIn(SigninResponse {
                    status: user.status,
                    name: user.name,
                    email: user.email,
                    user_id: user.id,
                    auth_token: hex::encode(auth_token),
                })))
            }
            _ => {
                ErrorType::BadRequest.res_err()
            }
        }
    })
}
