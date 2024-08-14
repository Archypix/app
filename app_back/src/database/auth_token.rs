use crate::database::database::DBConn;
use crate::database::schema::*;
use crate::database::utils::is_error_duplicate_key;
use crate::utils::auth::DeviceInfo;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use crate::utils::utils::{random_code, random_token};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::{delete, QueryDsl};
use diesel::{insert_into, update, Identifiable, Insertable, Queryable, RunQueryDsl, Selectable};
use diesel::{ExpressionMethods, OptionalExtension};
use rocket::Request;

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
    pub(crate) fn insert_token_for_user(conn: &mut DBConn, user_id: &u32, device_info: &DeviceInfo, try_count: u8) -> Result<Vec<u8>, ErrorResponder> {
        let auth_token = random_token(32);

        insert_into(auth_tokens::table)
            .values((
                auth_tokens::dsl::user_id.eq(user_id),
                auth_tokens::dsl::token.eq(&auth_token),
                auth_tokens::dsl::device_string.eq(&device_info.device_string),
                auth_tokens::dsl::ip_address.eq(inet6_aton(&device_info.ip_address))
            ))
            .execute(conn)
            .map(|_| auth_token)
            .or_else(|e| {
                if is_error_duplicate_key(&e, "auth_tokens.PRIMARY") && try_count < 4 {
                    println!("Auth token already exists, trying again.");
                    return AuthToken::insert_token_for_user(conn, user_id, device_info, try_count + 1);
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
    pub fn get_auth_token_from_headers(request: &Request<'_>) -> Option<Vec<u8>> {
        request.headers().get_one("X-Auth-Token").map(|s| hex::decode(s).ok()).flatten()
    }
    pub fn clear_auth_tokens(conn: &mut DBConn, user_id: &u32) -> Result<(), ErrorResponder> {
        delete(auth_tokens::table)
            .filter(auth_tokens::dsl::user_id.eq(user_id))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| {
                ErrorType::DatabaseError("Failed to delete existing auth tokens".to_string(), e).to_responder()
            })
    }
}


#[derive(Queryable, Selectable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(primary_key(user_id, token))]
#[diesel(belongs_to(User))]
#[diesel(table_name = confirmations)]
pub struct Confirmation {
    pub user_id: u32,
    pub action: ConfirmationAction,
    pub used: bool,
    pub date: NaiveDateTime,
    pub token: Vec<u8>,
    pub code_token: Vec<u8>,
    pub code: u16,
    pub code_trials: u8,
    pub device_string: Option<String>,
    pub ip_address: Option<Vec<u8>>,
}

impl Confirmation {
    pub(crate) fn insert_confirmation(conn: &mut DBConn, user_id: u32, action: ConfirmationAction, device_info: &DeviceInfo, try_count: u8) -> Result<(Vec<u8>, Vec<u8>, u16), ErrorResponder> {
        let token = random_token(16);
        let code_token = random_token(16);
        let code = random_code(4) as u16;

        insert_into(confirmations::table)
            .values((
                confirmations::dsl::user_id.eq::<u32>(user_id),
                confirmations::dsl::action.eq(&action),
                confirmations::dsl::token.eq(&token),
                confirmations::dsl::code_token.eq(&code_token),
                confirmations::dsl::code.eq(&code),
                confirmations::dsl::device_string.eq(&device_info.device_string),
                confirmations::dsl::ip_address.eq(inet6_aton(&device_info.ip_address))
            ))
            .execute(conn)
            .map(|_| (token, code_token, code))
            .or_else(|e| {
                if (is_error_duplicate_key(&e, "confirmations.PRIMARY") || is_error_duplicate_key(&e, "confirmations.UQ_confirmations")) && try_count < 3 {
                    println!("Confirmation token already exists, trying again.");
                    return Confirmation::insert_confirmation(conn, user_id, action, device_info, try_count + 1);
                }
                Err(ErrorType::DatabaseError("Failed to insert confirmation".to_string(), e).to_responder())
            })
    }
    pub fn check_code_and_mark_as_used(conn: &mut DBConn, user_id: &u32, action: &ConfirmationAction, code_token: &Vec<u8>, code: &u16) -> Result<(), ErrorResponder> {
        let confirmation = confirmations::table
            .filter(confirmations::dsl::user_id.eq(user_id))
            .filter(confirmations::dsl::action.eq(action))
            .filter(confirmations::dsl::code_token.eq(code_token))
            .filter(confirmations::dsl::code.eq(code))
            .first::<Confirmation>(conn)
            .optional()
            .map_err(|e| {
                ErrorType::DatabaseError("Failed to get confirmation".to_string(), e).to_responder()
            })?;
        if let Some(confirmation) = confirmation {
            if confirmation.used {
                return Err(ErrorType::ConfirmationAlreadyUsed.to_responder());
            }
            confirmation.mark_as_used(conn)?;
            return Ok(());
        }
        Err(ErrorType::ConfirmationNotFound.to_responder())
    }
    pub fn mark_as_used(&self, conn: &mut DBConn) -> Result<(), ErrorResponder> {
        update(confirmations::table)
            .filter(confirmations::dsl::user_id.eq(&self.user_id))
            .filter(confirmations::dsl::action.eq(&self.action))
            .filter(confirmations::dsl::token.eq(&self.token))
            .set((
                confirmations::dsl::used.eq(true),
            ))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| {
                ErrorType::DatabaseError("Failed to mark confirmation as used".to_string(), e).to_responder()
            })
    }
    pub fn mark_all_as_used(conn: &mut DBConn, user_id: &u32, action: ConfirmationAction) -> Result<(), ErrorResponder> {
        update(confirmations::table)
            .filter(confirmations::dsl::user_id.eq(user_id))
            .filter(confirmations::dsl::action.eq(action))
            .set((
                confirmations::dsl::used.eq(true),
            ))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| {
                ErrorType::DatabaseError("Failed to mark all confirmations as used".to_string(), e).to_responder()
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
