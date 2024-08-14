use rocket::form::Shareable;

pub fn is_error_duplicate_key(error: &diesel::result::Error, key: &str) -> bool {
    use diesel::result::Error;
    use diesel::result::DatabaseErrorKind;

    if let Error::DatabaseError(kind, info) = error {
        if let DatabaseErrorKind::UniqueViolation = kind {
            // format is "Duplicate entry 'example@gmail.come' for key 'users.email'"
            let error_parts = info.message().split('\'').collect::<Vec<&str>>();

            return error_parts.len() > 3 && error_parts[3] == key;
        }
    }
    false
}
