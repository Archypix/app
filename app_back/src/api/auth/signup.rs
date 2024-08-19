use diesel::Connection;
use rocket::serde::{json::Json, Deserialize};
use serde::Serialize;
use std::env;
use validator::Validate;

use crate::database::auth_token::Confirmation;
use crate::database::database::DBPool;
use crate::database::schema::ConfirmationAction;
use crate::database::user::User;
use crate::mailing::mailer::send_rendered_email;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::ErrorResponder;
use crate::utils::utils::left_pad;
use crate::utils::validation::validate_input;
use crate::utils::validation::validate_password;
use crate::utils::validation::validate_user_name;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupData {
    #[validate(custom(function = validate_user_name))]
    name: String,
    #[validate(email(code = "email_invalid", message = "Invalid email"))]
    email: String,
    #[validate(custom(function = validate_password))]
    password: String,
}

#[derive(Serialize, Debug)]
pub struct SignupResponse {
    pub(crate) id: u32,
    pub(crate) code_token: String,
}

#[post("/auth/signup", data = "<data>")]
pub fn auth_signup(data: Json<SignupData>, db: &rocket::State<DBPool>, device_info: DeviceInfo) -> Result<Json<SignupResponse>, ErrorResponder> {
    validate_input(&data)?;
    let conn = &mut db.get().unwrap();

    conn.transaction(|conn| {
        // Inserting user
        let uid = User::create_user(conn, &data.name, &data.email, &data.password)?;

        // Inserting confirmation
        let (confirm_token, confirm_code_token, confirm_code) = Confirmation::insert_confirmation(conn, uid, ConfirmationAction::Signup, &device_info, 0)?;
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

        Ok(Json(SignupResponse {
            id: uid,
            code_token: hex::encode(confirm_code_token),
        }))
    })
}
