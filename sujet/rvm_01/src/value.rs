use crate::inner_prelude::*;
use std::rc::Rc; // Import nécessaire pour les paires
use std::{fmt::Display, str::FromStr};

// Modif: Suppression du #[derive(Debug)] pour implémenter Debug manuellement (évite la récursion sur les paires)
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64), // <-- ajout de la variante Float
    Bool(bool),
    Str(String),
    Pair(Option<Rc<(Value, Value)>>), // Modif: Ajout de la variante Pair pour les paires. None = Nil, Some = paire de valeurs
}

// Modif: Implémentation manuelle de Debug pour gérer les paires de manière itérative (évite la récursion infinie)
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "Int({})", i),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Str(s) => write!(f, "Str({:?})", s),
            Value::Pair(None) => write!(f, "Nil"),
            Value::Pair(Some(rc)) => {
                // Affichage itératif des paires imbriquées
                write!(f, "Pair(")?;
                let mut current = rc.clone();
                loop {
                    write!(f, "({:?}, ", current.0)?;
                    match &current.1 {
                        Value::Pair(None) => {
                            write!(f, "Nil)")?;
                            break;
                        }
                        Value::Pair(Some(next)) => {
                            current = next.clone();
                        }
                        other => {
                            write!(f, "{:?})", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
        }
    }
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

    // Modif: Nouvelle méthode pour accéder à une paire
    pub fn as_pair(&self) -> Option<&Option<Rc<(Value, Value)>>> {
        match self {
            Value::Pair(pair) => Some(pair),
            _ => None,
        }
    }

    // Modif: Nouvelle méthode pour vérifier le type d'une valeur (utilisée par IsConsType)
    pub fn is_type(&self, const_type: ConstType) -> bool {
        match (self, const_type) {
            (Value::Int(_), ConstType::Int) => true,
            (Value::Float(_), ConstType::Float) => true,
            (Value::Bool(_), ConstType::Bool) => true,
            (Value::Str(_), ConstType::String) => true,
            (Value::Pair(_), ConstType::Pair) => true,
            _ => false,
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
    Int = 0,    // 0
    Float,      // 1
    Bool,       // 2
    String = 10, // 10
    Pair = 20,  // Modif: Ajout du type Pair pour les paires
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
            // Modif: Affichage des paires de manière linéaire et itérative: (a, b, c, d) au lieu de (a, (b, (c, d)))
            Value::Pair(None) => write!(f, "Nil"),
            Value::Pair(Some(rc)) => {
                write!(f, "(")?;
                let mut current = rc.clone();
                let mut first = true;
                loop {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}", current.0)?;
                    
                    match &current.1 {
                        Value::Pair(None) => break,
                        Value::Pair(Some(next)) => {
                            current = next.clone();
                        }
                        other => {
                            write!(f, ", {}", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
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
            // Modif: Affichage linéaire des paires pour l'instruction Print
            Value::Pair(None) => write!(f, "Nil"),
            Value::Pair(Some(rc)) => {
                write!(f, "(")?;
                let mut current = rc.clone();
                let mut first = true;
                loop {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}", current.0.as_printable())?;
                    
                    match &current.1 {
                        Value::Pair(None) => break,
                        Value::Pair(Some(next)) => {
                            current = next.clone();
                        }
                        other => {
                            write!(f, ", {}", other.as_printable())?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl FromStr for Value {
    type Err = ValueParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Modif: Ajout du parsing de "Nil" pour les paires vides
        if s == "Nil" {
            Ok(Value::Pair(None))
        } else if let Ok(value) = bool::from_str(s) {
            Ok(Value::Bool(value))
        } else if let Ok(value) = i64::from_str(s) {
            Ok(Value::Int(value))
        } else if let Ok(value) = f64::from_str(s) {
            Ok(Value::Float(value))
        } else if s.starts_with('"') && s.ends_with('"') {
            let s = s[1..s.len() - 1].to_owned();
            Ok(Value::Str(s.replace("\\n", "\n").replace("\\t", "\t")))
        } else {
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