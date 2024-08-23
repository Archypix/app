use rocket::serde::json::Json;
use std::borrow::Cow;
use validator::{Validate, ValidationError};

use crate::utils::errors_catcher::{ErrorResponder, ErrorType};

pub fn validate_input<T: Validate>(data: &Json<T>) -> Result<(), ErrorResponder> {
    if let Err(errors) = data.validate() {
        let message = errors.field_errors().iter().map(|(field, errors)| {
            field.to_string() + ": " + &errors.iter().filter_map(|error| error.clone().message.map(|s| s.to_string())).collect::<Vec<String>>().join(", ")
        }).collect::<Vec<String>>().join(", ");

        return ErrorType::InvalidInput(message).res_err();
    }
    Ok(())
}

pub fn validate_user_name(value: &str) -> Result<(), ValidationError> {
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return Err(ValidationError::new("name_whitespace")
            .with_message(Cow::from("Name cannot start or end with whitespace")));
    }
    if value.len() < 5 || value.len() > 100 {
        return Err(ValidationError::new("name_length")
            .with_message(Cow::from("Name must be between 5 and 100 characters")));
    }
    Ok(())
}

pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    if value.len() < 8 || value.len() > 100 {
        return Err(ValidationError::new("password_length")
            .with_message(Cow::from("Password must be between 8 and 100 characters")));
    }
    if !value.chars().any(|c| c.is_ascii_lowercase())
        || !value.chars().any(|c| c.is_ascii_uppercase())
        || !value.chars().any(|c| c.is_ascii_digit())
    {
        return Err(ValidationError::new("password_requirements")
            .with_message(Cow::from("Password must contain at least one lowercase letter, one uppercase letter and one digit")));
    }

    Ok(())
}
