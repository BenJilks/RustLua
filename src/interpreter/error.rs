use core::fmt;

use super::Value;

pub enum LuaError {
    InvalidIndex(Value),
    InvalidCall(Value),
    InvalidArithmetic(Value),
}

impl fmt::Debug for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIndex(v) => write!(f, "attempt to index a {} value", v.type_name()),
            Self::InvalidCall(v) => write!(f, "attempt to call a {} value", v.type_name()),
            Self::InvalidArithmetic(v) => write!(f, "attempt to perform arithmetic on a {} value", v.type_name()),
        }
    }
}
