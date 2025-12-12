use crate::inner_prelude::*;

/// Here are all the different Error Type that we use in the tool

#[derive(Debug, Clone)]
pub enum ValueParsingError {
    UnknownValue(String),
}

#[derive(Debug, Clone)]
pub enum InstructionParsingError {
    UnknownInstruction {
        invalid_operator: String,
    },
    InvalidArg {
        column: usize,
        arg: String,
    },
    UnknownConst {
        column: usize,
        invalid_const: String,
    },
}

#[derive(Debug, Clone)]
pub enum FileParsingError {
    UnknownInstruction {
        location: Location,
        line: String,
        invalid_instruction: String,
    },
    UnknownConst {
        location: Location,
        line: String,
        invalid_const: String,
    },
    InvalidArg {
        location: Location,
        line: String,
        invalid_arg: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum ContextUpdateError {
    HaltExecution,
    TypeError {
        op_num: usize,
        operand: Value,
        expected_value: ConstType,
    },
    MissingOperand {
        ops_found: usize,
        ops_needed: usize,
    },
    RegOutOfIndex {
        reg_index: usize,
    },
    // MODIF: Nouvelle variante pour les erreurs d'accès aux paires
    InvalidPairAccess {
        pair_id: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

// MODIF: Correction de la définition de l'enum ExecutionError
// La syntaxe était invalide (mélange de définition et de code d'affichage)
#[derive(Debug, Clone)]
pub enum ExecutionError {
    TypeError {
        location: Location,
        instruction: Instruction,
        op_num: usize,
        operand: Value,
        expected_type: ConstType,
    },
    MissingOperand {
        location: Location,
        instruction: Instruction,
        ops_found: usize,
        ops_needed: usize,
    },
    RegOutOfIndex {
        location: Location,
        instruction: Instruction,
        stack_len: usize,
        reg_index: usize,
    },
    // MODIF: Nouvelle variante pour les erreurs d'accès aux paires en mémoire
    // Syntaxe corrigée : c'était une définition d'enum, pas du code exécutable
    InvalidPairAccess {
        location: Location,
        instruction: Instruction,
        pair_id: usize,
    },
}

// MODIF: Ajout de l'affichage pour InvalidPairAccess
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::TypeError {
                location,
                instruction,
                op_num,
                operand,
                expected_type,
            } => write!(
                f,
                "Type error at {}:{} in instruction '{}': operand {} has value {:?} but expected type {:?}",
                location.line, location.column, instruction, op_num, operand, expected_type
            ),
            ExecutionError::MissingOperand {
                location,
                instruction,
                ops_found,
                ops_needed,
            } => write!(
                f,
                "Missing operand at {}:{} in instruction '{}': found {} operand(s) but needed {}",
                location.line, location.column, instruction, ops_found, ops_needed
            ),
            ExecutionError::RegOutOfIndex {
                location,
                instruction,
                stack_len,
                reg_index,
            } => write!(
                f,
                "Register out of bounds at {}:{} in instruction '{}': tried to access register {} but stack has only {} elements",
                location.line, location.column, instruction, reg_index, stack_len
            ),
            // MODIF: Affichage de l'erreur InvalidPairAccess
            ExecutionError::InvalidPairAccess {
                location,
                instruction,
                pair_id,
            } => write!(
                f,
                "Invalid pair access at {}:{} in instruction '{}': tried to access pair with id {} but it doesn't exist or was freed",
                location.line, location.column, instruction, pair_id
            ),
        }
    }
}

impl std::error::Error for ExecutionError {}