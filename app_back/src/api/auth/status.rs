use rocket::serde::json::Json;
use rocket::serde::Serialize;

use crate::database::schema::UserStatus;
use crate::database::user::User;
use crate::utils::errors_catcher::ErrorResponder;

#[derive(Serialize, Debug)]
pub struct StatusResponse {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) status: UserStatus,
}

#[get("/auth/status")]
pub fn auth_status(user: Result<User, ErrorResponder>) -> Result<Json<StatusResponse>, ErrorResponder> {
    let user = user?;
    Ok(Json(StatusResponse {
        name: user.name,
        email: user.email,
        status: user.status,
    }))
}
