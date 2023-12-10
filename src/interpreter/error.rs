use core::fmt;
use std::error::Error;

use super::Value;

#[derive(PartialEq, Debug)]
pub enum LuaError {
    InvalidIndex(Value),
    InvalidCall(Value),
    InvalidArithmetic(Value),
    BadForLimit(Value),
    BadForInitialValue(Value),
    BadForStep(Value),
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIndex(v) => write!(f, "attempt to index a {} value", v.type_name()),
            Self::InvalidCall(v) => write!(f, "attempt to call a {} value", v.type_name()),
            Self::InvalidArithmetic(v) => write!(f, "attempt to perform arithmetic on a {} value", v.type_name()),
            Self::BadForLimit(v) => write!(f, "bad 'for' limit (number expected, got {})", v.type_name()),
            Self::BadForInitialValue(v) => write!(f, "bad 'for' initial value (number expected, got {})", v.type_name()),
            Self::BadForStep(v) => write!(f, "bad 'for' step (number expected, got {})", v.type_name()),
        }
    }
}

impl Error for LuaError {}
