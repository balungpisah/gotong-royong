use crate::error::ApiError;
use validator::Validate;

pub fn validate<T: Validate>(value: &T) -> Result<(), ApiError> {
    value
        .validate()
        .map_err(|err| ApiError::Validation(err.to_string()))?;
    Ok(())
}
