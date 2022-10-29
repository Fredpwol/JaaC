use std::error;

#[derive(Debug, Clone, Copy)]
pub struct LoginError;


impl std::fmt::Display for LoginError{

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to login user")
    }
}

impl error::Error for LoginError {}
