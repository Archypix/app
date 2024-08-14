use crate::database::database::DBConn;
use crate::database::schema::*;
use crate::database::utils::is_error_duplicate_key;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use crate::utils::utils::{random_code, random_token};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::ExpressionMethods;
use diesel::{insert_into, update, Identifiable, Insertable, Queryable, RunQueryDsl, Selectable};

#[derive(Queryable, Selectable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(primary_key(user_id, token))]
#[diesel(belongs_to(User))]
#[diesel(table_name = auth_tokens)]
pub struct AuthToken {
    pub user_id: u32,
    pub token: Vec<u8>,
    pub creation_date: NaiveDateTime,
    pub last_use_date: NaiveDateTime,
    pub device_string: Option<String>,
    pub ip_address: Option<Vec<u8>>,
}

impl AuthToken {
    pub(crate) fn insert_token_for_user(conn: &mut DBConn, user_id: u32, device_info: &DeviceInfo) -> Result<Vec<u8>, ErrorResponder> {
        let auth_token = random_token(32);
        insert_into(auth_tokens::table)
            .values((
                auth_tokens::dsl::user_id.eq::<u32>(user_id),
                auth_tokens::dsl::token.eq(&auth_token),
                auth_tokens::dsl::device_string.eq(&device_info.device_string),
                auth_tokens::dsl::ip_address.eq(inet6_aton(&device_info.ip_address))
            ))
            .execute(conn)
            .map(|_| auth_token)
            .or_else(|e| {
                if is_error_duplicate_key(&e, "auth_tokens.token") {
                    println!("Auth token already exists, trying again.");
                    return AuthToken::insert_token_for_user(conn, user_id, device_info);
                }
                Err(ErrorType::DatabaseError("Failed to insert auth token".to_string(), e).to_responder())
            })
    }
    pub fn update_last_use_date(&self, conn: &mut DBConn) -> Result<(), ErrorResponder> {
        // Working in UTC time.
        let current_naive = Utc::now().naive_utc();
        if current_naive - self.last_use_date > TimeDelta::try_minutes(10).unwrap() {
            println!("Updating last_use_date");
            update(auth_tokens::table)
                .filter(auth_tokens::dsl::user_id.eq(self.user_id))
                .filter(auth_tokens::dsl::token.eq(self.token.clone()))
                .set((
                    auth_tokens::dsl::last_use_date.eq(utc_timestamp()),
                ))
                .execute(conn).map_err(|e| {
                ErrorType::DatabaseError("Failed to update auth token use date".to_string(), e).to_responder()
            })?;
        }
        Ok(())
    }
}


#[derive(Queryable, Selectable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(primary_key(user_id, token))]
#[diesel(belongs_to(User))]
#[diesel(table_name = confirmations)]
pub struct Confirmation {
    pub user_id: u32,
    pub action: ConfirmationAction,
    pub date: NaiveDateTime,
    pub token: Vec<u8>,
    pub code: u16,
    pub code_trials: u8,
    pub device_string: Option<String>,
    pub ip_address: Option<Vec<u8>>,
}

impl Confirmation {
    pub(crate) fn insert_confirmation(conn: &mut DBConn, user_id: u32, action: ConfirmationAction, device_info: &DeviceInfo) -> Result<(Vec<u8>, u16), ErrorResponder> {
        let token = random_token(16);
        let code = random_code(4) as u16;

        insert_into(confirmations::table)
            .values((
                confirmations::dsl::user_id.eq::<u32>(user_id),
                confirmations::dsl::token.eq(&token),
                confirmations::dsl::action.eq(&action),
                confirmations::dsl::code.eq(&code),
                confirmations::dsl::device_string.eq(&device_info.device_string),
                confirmations::dsl::ip_address.eq(inet6_aton(&device_info.ip_address))
            ))
            .execute(conn)
            .map(|_| (token, code))
            .or_else(|e| {
                if is_error_duplicate_key(&e, "confirmations.token") {
                    println!("Confirmation token already exists, trying again.");
                    return Confirmation::insert_confirmation(conn, user_id, action, device_info);
                }
                Err(ErrorType::DatabaseError("Failed to insert confirmation".to_string(), e).to_responder())
            })
    }
}

#[derive(Queryable, Selectable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(table_name = totp_secrets)]
pub struct TOTPSecret {
    pub user_id: u32,
    pub creation_date: NaiveDateTime,
    pub secret: Vec<u8>,
}
