use diesel::result::Error;
use enum_kinds::EnumKind;
use rocket::serde::json::Json;
use rocket::Request;
use serde::Serialize;

#[derive(Responder, Debug)]
pub enum ErrorResponder {
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponse>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorResponse>),
    #[response(status = 422, content_type = "json")]
    UnprocessableEntity(Json<ErrorResponse>),
    #[response(status = 500, content_type = "json")]
    InternalError(Json<ErrorResponse>),
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error_type: ErrorTypeKind,
    pub message: String,
}

#[derive(EnumKind, Debug)]
#[enum_kind(ErrorTypeKind, derive(Serialize))]
pub enum ErrorType {
    BadRequest,
    Unauthorized,
    NotFound(String),
    UnprocessableEntity,
    InternalError,
    // Form validation (see UnprocessableEntity for type check related errors)
    InvalidInput(String),
    // User request guard
    UserNotFound,
    UserBanned,
    UserUnconfirmed,
    // Sign up types
    EmailAlreadyExists,
    // Confirm
    ConfirmationAlreadyUsed,
    ConfirmationNotFound,
    // Admin
    UserNotAdmin,
    // Database error
    DatabaseError(String, Error),
}

impl ErrorType {
    pub fn to_err<T>(self) -> Result<T, ErrorResponder> {
        Err(self.to_responder())
    }
    pub fn to_responder(self) -> ErrorResponder {
        let kind = ErrorTypeKind::from(&self);
        match self {
            // Default HTTP types
            ErrorType::BadRequest => ErrorResponder::BadRequest(Self::res("Bad request".to_string(), kind)),
            ErrorType::Unauthorized => ErrorResponder::Unauthorized(Self::res("Unauthorized".to_string(), kind)),
            ErrorType::NotFound(path) => ErrorResponder::NotFound(Self::res(format!("Not found: {}", path), kind)),
            ErrorType::UnprocessableEntity => ErrorResponder::UnprocessableEntity(Self::res("Unprocessable entity".to_string(), kind)),
            ErrorType::InternalError => ErrorResponder::InternalError(Self::res("Internal error".to_string(), kind)),
            // Form validation (see UnprocessableEntity for type check related errors)
            ErrorType::InvalidInput(msg) => ErrorResponder::UnprocessableEntity(Self::res(msg, kind)),
            // Sign in / status types
            ErrorType::UserNotFound => ErrorResponder::Unauthorized(Self::res("User not found".to_string(), kind)),
            ErrorType::UserBanned => ErrorResponder::Unauthorized(Self::res("User is banned".to_string(), kind)),
            ErrorType::UserUnconfirmed => ErrorResponder::Unauthorized(Self::res("User is not confirmed".to_string(), kind)),
            // Sign up types
            ErrorType::EmailAlreadyExists => ErrorResponder::Unauthorized(Self::res("Email already exists".to_string(), kind)),
            // Confirm
            ErrorType::ConfirmationAlreadyUsed => ErrorResponder::Unauthorized(Self::res("Confirmation code/token already used".to_string(), kind)),
            ErrorType::ConfirmationNotFound => ErrorResponder::Unauthorized(Self::res("Invalid code/token".to_string(), kind)),
            // Admin
            ErrorType::UserNotAdmin => ErrorResponder::Unauthorized(Self::res("User is not an admin".to_string(), kind)),
            // Database error
            ErrorType::DatabaseError(msg, err) => ErrorResponder::InternalError(Self::res(format!("Database error: {} - {}", msg, err), kind)),
        }
    }
    fn res(msg: String, kind: ErrorTypeKind) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            message: msg,
            error_type: kind,
        })
    }
}

impl From<Error> for ErrorResponder {
    fn from(value: Error) -> Self {
        ErrorType::DatabaseError("Diesel error".to_string(), value).to_responder()
    }
}


#[catch(400)]
pub fn bad_request() -> ErrorResponder {
    ErrorType::BadRequest.to_responder()
}
#[catch(401)]
pub fn unauthorized() -> ErrorResponder {
    ErrorType::Unauthorized.to_responder()
}
#[catch(404)]
pub fn not_found(req: &Request) -> ErrorResponder {
    ErrorType::NotFound(req.uri().to_string()).to_responder()
}
/// When a JSON value type is incorrect
#[catch(422)]
pub fn unprocessable_entity() -> ErrorResponder {
    ErrorType::UnprocessableEntity.to_responder()
}
#[catch(500)]
pub fn internal_error() -> ErrorResponder {
    ErrorType::InternalError.to_responder()
}
