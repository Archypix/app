use crate::database::database::DBConn;
use crate::database::schema::*;
use crate::database::utils::is_error_duplicate_key;
use crate::utils::errors_catcher::{ErrorResponder, ErrorType};
use chrono::NaiveDateTime;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{insert_into, select, update, Associations, Identifiable, Insertable, OptionalExtension, Queryable, RunQueryDsl, Selectable};
use pwhash::bcrypt;

#[derive(Queryable, Selectable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(table_name = users)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub creation_date: NaiveDateTime,
    pub status: UserStatus,
    pub storage_count_ko: u64,
    pub storage_limit_mo: u32,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(primary_key(user_id_acceptor, user_id_sharer))]
#[diesel(belongs_to(User, foreign_key = user_id_acceptor, foreign_key = user_id_sharer))]
#[diesel(table_name = shares_auto_accept)]
pub struct ShareAutoAccept {
    pub user_id_acceptor: u32,
    pub user_id_sharer: u32,
}

impl User {
    pub(crate) fn create_user(conn: &mut DBConn, name: &str, email: &str, password: &str) -> Result<u32, ErrorResponder> {
        // Check if the user exists and update only if status is unconfirmed
        let existing_user: Option<User> = users::table
            .filter(users::dsl::email.eq(email))
            .first(conn)
            .optional()
            .map_err(|e| {
                ErrorType::DatabaseError("Failed to get already existing user".to_string(), e).to_responder()
            })?;

        if let Some(user) = existing_user {
            if user.status != UserStatus::Unconfirmed {
                return Err(ErrorType::EmailAlreadyExists.to_responder());
            }
            update(users::table)
                .filter(users::dsl::id.eq(user.id))
                .set((
                    users::dsl::name.eq::<String>(name.to_string()),
                    users::dsl::password_hash.eq(bcrypt::hash(password).unwrap()),
                    users::dsl::creation_date.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)
                .map_err(|e| {
                    ErrorType::DatabaseError("Failed to update user name and password.".to_string(), e).to_responder()
                })?;

            // Remove all existing confirmations
            diesel::delete(confirmations::table.filter(confirmations::dsl::user_id.eq(user.id)))
                .execute(conn)
                .map_err(|e| {
                    ErrorType::DatabaseError("Failed to delete existing confirmations".to_string(), e).to_responder()
                })?;
            // Remove all existing auth tokens
            diesel::delete(auth_tokens::table.filter(auth_tokens::dsl::user_id.eq(user.id)))
                .execute(conn)
                .map_err(|e| {
                    ErrorType::DatabaseError("Failed to delete existing auth tokens".to_string(), e).to_responder()
                })?;

            return Ok(user.id);
        }

        insert_into(users::table)
            .values((
                users::dsl::name.eq::<String>(name.to_string()),
                users::dsl::email.eq(email.to_string()),
                users::dsl::password_hash.eq(bcrypt::hash(password).unwrap()),
            ))
            .execute(conn)
            .map_err(|e| {
                if is_error_duplicate_key(&e, "users.email") {
                    return ErrorType::EmailAlreadyExists.to_responder();
                }
                ErrorType::DatabaseError("Failed to insert user".to_string(), e).to_responder()
            })
            .and_then(|result| {
                select(last_insert_id()).get_result::<u64>(conn)
                    .map(|id| id as u32)
                    .map_err(|e| {
                        ErrorType::DatabaseError("Failed to get last insert id".to_string(), e).to_responder()
                    })
            })
    }
}

impl ShareAutoAccept {}
