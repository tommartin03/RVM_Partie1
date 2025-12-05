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
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

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
}
