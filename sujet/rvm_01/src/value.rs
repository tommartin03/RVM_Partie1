use crate::inner_prelude::*;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl Default for Value {
    fn default() -> Self {
        Value::Int(0)
    }
}

impl Value {
    pub fn to_int(&self) -> Option<i64> {
        match self {
            &Value::Int(value) => Some(value),
            _ => None,
        }
    }

    pub fn to_float(&self) -> Option<f64> {
        match self {
            &Value::Float(value) => Some(value),
            &Value::Int(value) => Some(value as f64),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            &Value::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_printable(&self) -> PrintDisplay {
        PrintDisplay(self)
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ConstType {
    Int = 0, // 0
    Float,   // 1
    Bool,    // 2

    String = 10, // 10

    IntI8 = 200, // 200
    IntI16,      // 201
    IntI32,      // 202
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => Display::fmt(value, f),
            Value::Float(value) => Display::fmt(value, f),
            Value::Bool(value) => Display::fmt(value, f),
            Value::Str(value) => Display::fmt(&value.replace("\n", "\\n").replace("\t", "\\t"), f),
        }
    }
}

pub(crate) struct PrintDisplay<'a>(&'a Value);

impl<'a> Display for PrintDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Int(value) => Display::fmt(value, f),
            Value::Float(value) => Display::fmt(value, f),
            Value::Bool(value) => Display::fmt(value, f),
            Value::Str(value) => Display::fmt(value, f),
        }
    }
}

impl FromStr for Value {
    type Err = ValueParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = bool::from_str(s) {
            Ok(Value::Bool(value))
        } else if let Ok(value) = i64::from_str(s) {
            Ok(Value::Int(value))
        } else if let Ok(value) = f64::from_str(s) {
            Ok(Value::Float(value))
        } else if s.starts_with('"') && s.ends_with('"') {
            let s = s[1..s.len() - 1].to_owned();
            Ok(Value::Str(s.replace("\\n", "\n").replace("\\t", "\t")))
        }
        else {
            Err(ValueParsingError::UnknownValue(s.to_owned()))
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Str(value)
    }
}
