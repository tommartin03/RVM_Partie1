use crate::inner_prelude::*;
use std::{fmt::Display, str::FromStr};

// MODIF: Ajout du type PairId pour référencer une paire dans la mémoire
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairId(pub usize);

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64), // <-- ajout de la variante Float
    Bool(bool),
    Str(String),
    // MODIF: Ajout de la variante Pair qui contient un PairId (référence vers la mémoire)
    Pair(PairId),
    // MODIF: Ajout de la variante Nil pour représenter une paire vide
    Nil,
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

    // MODIF: Ajout d'une méthode pour extraire le PairId
    pub fn to_pair_id(&self) -> Option<PairId> {
        match self {
            Value::Pair(id) => Some(*id),
            _ => None,
        }
    }

    // MODIF: Ajout d'une méthode pour vérifier si la valeur est Nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub(crate) fn as_printable(&self) -> PrintDisplay<'_> {
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
    
    // MODIF: Ajout du type Nil
    Nil = 20, // 20
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => Display::fmt(value, f),
            Value::Float(value) => Display::fmt(value, f),
            Value::Bool(value) => Display::fmt(value, f),
            Value::Str(value) => Display::fmt(&value.replace("\n", "\\n").replace("\t", "\\t"), f),
            // MODIF: Affichage des paires et de Nil
            Value::Pair(id) => write!(f, "Pair({})", id.0),
            Value::Nil => write!(f, "Nil"),
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
            // MODIF: Affichage des paires et de Nil pour Print
            Value::Pair(id) => write!(f, "Pair({})", id.0),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

impl FromStr for Value {
    type Err = ValueParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // MODIF: Ajout du parsing pour Nil
        if s == "Nil" {
            Ok(Value::Nil)
        } else if let Ok(value) = bool::from_str(s) {
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