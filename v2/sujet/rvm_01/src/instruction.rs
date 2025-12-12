use crate::inner_prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{fmt::Display, str::FromStr};

// Here, we modelize our instructions an enum type

/// Represents the index of a register (stack of the program)
#[derive(Debug, Clone, Copy)]
pub struct RegIdx(pub u32);

/// Represents a program address (index of an instruction)
#[derive(Debug, Clone, Copy)]
pub enum Addr {
    InstructionIdx(u32),
}

impl Addr {
    pub fn to_idx(&self) -> usize {
        match self {
            &Addr::InstructionIdx(idx) => idx as usize,
        }
    }

    pub fn increment(&mut self) {
        match self {
            Addr::InstructionIdx(idx) => *idx += 1,
        }
    }

}
/// The instruction type
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,

    And,
    Or,
    Not,

    Push,
    Pop,

    Get(RegIdx),
    Set(RegIdx),

    // Modif: Nouvelles instructions combinées
    PushReg(RegIdx), // Combine Get + Push
    PopSet(RegIdx),  // Combine Pop + Set

    Print,

    Jump(Addr),
    Call(Addr),
    Branch(Addr),
    Ret,

    Const(Value),

    // MODIF: Ajout de l'instruction Pair pour créer une paire (car, cdr)
    Pair,
    // MODIF: Ajout de l'instruction Car pour extraire le premier élément d'une paire
    Car,
    // MODIF: Ajout de l'instruction Cdr pour extraire le second élément d'une paire
    Cdr,
    
    // MODIF: Ajout des instructions Is, First, Second
    Is(ConstType),  // Vérifie le type d'une valeur (Is Pair, Is Int, etc.)
    First,          // Alias pour Car - extrait le premier élément d'une paire
    Second,         // Alias pour Cdr - extrait le second élément d'une paire

    Noop,
    Halt,
}

// Those operation take classes of instructions and returns modified version, it is usefull to regroup program behavior on a per class basis.
impl Instruction {
    pub fn with_value(&self, value: Value) -> Option<Instruction> {
        use Instruction as I;
        match self {
            I::Const(..) => Some(I::Const(value)),
            _ => None,
        }
    }

    pub fn with_reg(&self, reg_idx: RegIdx) -> Option<Instruction> {
        use Instruction as I;
        match self {
            I::Get(..) => Some(I::Get(reg_idx)),
            I::Set(..) => Some(I::Set(reg_idx)),
            I::PushReg(..) => Some(I::PushReg(reg_idx)), // <--- ajouté ici
            I::PopSet(..) => Some(I::PopSet(reg_idx)),   // <--- ajouté ici
            _ => None,
        }
    }

    pub fn with_address(&self, addr: Addr) -> Option<Instruction> {
        use Instruction as I;
        match self {
            I::Jump(..) => Some(I::Jump(addr)),
            I::Call(..) => Some(I::Call(addr)),
            I::Branch(..) => Some(I::Branch(addr)),
            _ => None,
        }
    }
}

// Here, we parse an instruction
impl FromStr for Instruction {
    type Err = InstructionParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction as I;
        // we remove a possible comment at the end of the line (split), and then we remove extra spaces / tabs / ... (trim)
        let trimmed_s = s.split('#').next().unwrap().trim();
        // We can match strings in Rust :)
        Ok(match trimmed_s {
            "" | "Noop" => I::Noop,
            "Add" => I::Add,
            "Sub" => I::Sub,
            "Mul" => I::Mul,
            "Div" => I::Div,
            "Lt" => I::Lt,
            "Le" => I::Le,
            "And" => I::And,
            "Or" => I::Or,
            "Not" => I::Not,
            "Push" => I::Push,
            "Pop" => I::Pop,
            "Print" => I::Print,
            "Ret" => I::Ret,
            "Halt" => I::Halt,
            // MODIF: Ajout du parsing pour les instructions Pair, Car, Cdr
            "Pair" => I::Pair,
            "Car" => I::Car,
            "Cdr" => I::Cdr,
            // MODIF: Ajout du parsing pour First et Second
            "First" => I::First,
            "Second" => I::Second,
            _ => {
                let (operator, args) = trimmed_s
                    .split_once(|c: char| c.is_whitespace())
                    .ok_or_else(|| InstructionParsingError::InvalidArg {
                        column: 1,
                        arg: String::new(),
                    })?;
                let args = args.trim();

                match operator {
                    "Get" | "Set" | "PushReg" | "PopSet" => {
                        let reg_index = args.parse().or_else(|_| {
                            Err(InstructionParsingError::InvalidArg {
                                column: 1,
                                arg: args.to_owned(),
                            })
                        })?;
                        match operator {
                            "Get" => I::Get(RegIdx(reg_index)),
                            "Set" => I::Set(RegIdx(reg_index)),
                            "PushReg" => I::PushReg(RegIdx(reg_index)), // <--- nouveau
                            "PopSet" => I::PopSet(RegIdx(reg_index)),   // <--- nouveau
                            _ => unreachable!(),
                        }
                    }

                    "Jump" | "Call" | "Branch" => {
                        let addr: u32 = args.parse().or_else(|_| {
                            Err(InstructionParsingError::InvalidArg {
                                column: 1,
                                arg: args.to_owned(),
                            })
                        })?;
                        match operator {
                            "Jump" => I::Jump(Addr::InstructionIdx(addr - 1)),
                            "Call" => I::Call(Addr::InstructionIdx(addr - 1)),
                            "Branch" => I::Branch(Addr::InstructionIdx(addr - 1)),
                            _ => unreachable!(),
                        }
                    }

                    // MODIF: Ajout du parsing pour "Is"
                    "Is" => {
                        let const_type = match args {
                            "Pair" => ConstType::Nil,  // On utilise Nil pour vérifier les paires
                            "Int" => ConstType::Int,
                            "Float" => ConstType::Float,
                            "Bool" => ConstType::Bool,
                            "String" => ConstType::String,
                            "Nil" => ConstType::Nil,
                            _ => {
                                return Err(InstructionParsingError::InvalidArg {
                                    column: operator.len() + 2,
                                    arg: args.to_owned(),
                                })
                            }
                        };
                        I::Is(const_type)
                    }

                    "Const" => {
                        let value = args.parse().map_err(|err| match err {
                            ValueParsingError::UnknownValue(invalid_const) => {
                                InstructionParsingError::UnknownConst {
                                    column: 1,
                                    invalid_const,
                                }
                            }
                        })?;
                        I::Const(value)
                    }

                    op => {
                        return Err(InstructionParsingError::UnknownInstruction {
                            invalid_operator: op.to_owned(),
                        })
                    }
                }
            }
        })
    }
}

// Here, we reconvert instruction to their text assembly format
impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction as I;
        match self {
            I::Noop => write!(f, "Noop"),
            I::Add => write!(f, "Add"),
            I::Sub => write!(f, "Sub"),
            I::Mul => write!(f, "Mul"),
            I::Div => write!(f, "Div"),
            I::Lt => write!(f, "Lt"),
            I::Le => write!(f, "Le"),
            I::And => write!(f, "And"),
            I::Or => write!(f, "Or"),
            I::Not => write!(f, "Not"),
            I::Push => write!(f, "Push"),
            I::Pop => write!(f, "Pop"),
            I::Print => write!(f, "Print"),
            I::Ret => write!(f, "Ret"),
            I::Halt => write!(f, "Halt"),
            I::Get(idx) => write!(f, "Get {}", idx.0),
            I::Set(idx) => write!(f, "Set {}", idx.0),
            I::PushReg(idx) => write!(f, "PushReg {}", idx.0), // <--- affichage ajouté
            I::PopSet(idx) => write!(f, "PopSet {}", idx.0),   // <--- affichage ajouté
            I::Jump(Addr::InstructionIdx(idx)) => write!(f, "Jump {}", idx + 1),
            I::Call(Addr::InstructionIdx(idx)) => write!(f, "Call {}", idx + 1),
            I::Branch(Addr::InstructionIdx(idx)) => write!(f, "Branch {}", idx + 1),
            I::Const(Value::Str(value)) => write!(
                f,
                "Const \"{}\"",
                value.replace("\n", "\\n").replace("\t", "\\t")
            ),
            I::Const(value) => write!(f, "Const {}", value),
            // MODIF: Ajout de l'affichage pour Pair, Car, Cdr
            I::Pair => write!(f, "Pair"),
            I::Car => write!(f, "Car"),
            I::Cdr => write!(f, "Cdr"),
            // MODIF: Ajout de l'affichage pour Is, First, Second
            I::Is(const_type) => {
                let type_name = match const_type {
                    ConstType::Nil => "Pair",  // "Is Pair" vérifie si c'est une paire
                    ConstType::Int => "Int",
                    ConstType::Float => "Float",
                    ConstType::Bool => "Bool",
                    ConstType::String => "String",
                    _ => "Unknown",
                };
                write!(f, "Is {}", type_name)
            }
            I::First => write!(f, "First"),
            I::Second => write!(f, "Second"),
        }
    }
}

impl Instruction {
    /// Parses a test assembly file and returns all its instruction
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Instruction>, FileParsingError> {
        let file = File::open(path).unwrap();
        BufReader::new(file)
            .lines()
            .enumerate()
            .map(|(idx, line_res)| {
                let line = line_res.unwrap();
                line.trim().parse().map_err(move |err| (idx, line, err))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|(line_idx, line, err)| match err {
                InstructionParsingError::UnknownConst {
                    column,
                    invalid_const,
                } => FileParsingError::UnknownConst {
                    location: Location {
                        line: line_idx + 1,
                        column,
                    },
                    line,
                    invalid_const,
                },
                InstructionParsingError::InvalidArg { column, arg } => FileParsingError::InvalidArg {
                    location: Location {
                        line: line_idx + 1,
                        column,
                    },
                    line,
                    invalid_arg: arg,
                },
                InstructionParsingError::UnknownInstruction { invalid_operator } => {
                    FileParsingError::UnknownInstruction {
                        location: Location {
                            line: line_idx + 1,
                            column: 1,
                        },
                        line,
                        invalid_instruction: invalid_operator,
                    }
                }
            })
    }
}