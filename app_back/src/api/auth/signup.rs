use diesel::result::DatabaseErrorKind;
use diesel::{insert_into, select, ExpressionMethods, RunQueryDsl};
use pwhash::bcrypt;
use rocket::serde::{json::Json, Deserialize};
use serde::Serialize;
use std::env;
use validator::Validate;

use crate::database::auth_token::{AuthToken, Confirmation};
use crate::database::database::DBPool;
use crate::database::schema::{last_insert_id, users::dsl::*, ConfirmationAction};
use crate::database::user::User;
use crate::database::utils::is_error_duplicate_key;
use crate::mailing::mailer::send_rendered_email;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use crate::utils::utils::{left_pad, random_token};
use crate::utils::validation::validate_input;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupData {
    #[validate(length(min = 3, max = 100, message = "Length must be between 3 and 100 characters"))]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 100, message = "Length must be between 8 and 100 characters"))]
    password: String,
}

#[derive(Serialize, Debug)]
pub struct SignupResponse {
    pub(crate) user_id: u32,
    pub(crate) auth_token: String,
}

#[post("/auth/signup", data = "<data>")]
pub fn auth_signup(data: Json<SignupData>, db: &rocket::State<DBPool>, device_info: DeviceInfo) -> Result<Json<SignupResponse>, ErrorResponder> {
    validate_input(&data)?;
    let conn = &mut db.get().unwrap();

    // Inserting user
    let uid = User::create_user(conn, &data.name, &data.email, &data.password)?;

    // Inserting confirmation
    let (confirm_token, confirm_code) = Confirmation::insert_confirmation(conn, uid, ConfirmationAction::Signup, &device_info)?;
    let confirm_code_str = left_pad(&confirm_code.to_string(), '0', 4);

    // Sending email
    let signup_url = format!("{}/signup/confirm?id={}&token={}",
                             env::var("FRONTEND_HOST").expect("FRONTEND_HOST must be set"), uid, hex::encode(&confirm_token));
    let subject = "Confirm your email address".to_string();
    let mut context = tera::Context::new();
    context.insert("name", &data.name);
    context.insert("url", &signup_url);
    context.insert("code", &confirm_code_str);
    send_rendered_email((data.name.clone(), data.email.clone()), subject, "confirm_signup".to_string(), context);

    // Inserting auth token & returning
    let auth_token = AuthToken::insert_token_for_user(conn, uid, &device_info)?;
    Ok(Json(SignupResponse {
        user_id: uid,
        auth_token: hex::encode(auth_token),
    }))
}
