use std::ffi::{OsStr, OsString};
use std::process::ExitCode;

use colored::Colorize;

use clap::Parser;
use rvm::cli::{Cli, Command};
use rvm::error::{ExecutionError, FileParsingError, Location};
use rvm::instruction::Instruction;
use rvm::vm::OpVM;

fn print_exit_on_error(
    path: &OsStr,
    parsing_error: bool,
    location: Location,
    line: &str,
    error_message: &str,
) {
    if parsing_error {
        eprint!("{}", "Parsing Error: ".red());
    } else {
        eprint!("{}", "Execution Error: ".red())
    }

    if let Some(path) = path.to_str() {
        eprint! {"{}", path};
    } else {
        eprint! {"{:?}", path};
    }

    eprintln!(":{}:{}", location.line, location.column);

    eprintln!("{}", "    |".cyan());
    eprintln!("{}  {}", format!("{:>3} |", location.line).cyan(), line);
    eprintln!(
        "{}  {}{} {}",
        "    |".cyan(),
        " ".repeat(location.column - 1),
        "^".repeat(line.chars().count() + 1 - location.column).red(),
        error_message.red()
    );
    eprintln!("{}", "    |".cyan());
}

fn handle_parse_result(path: &OsStr) -> Option<Vec<Instruction>> {
    Instruction::parse_file(&path)
        .map_err(|err| match err {
            FileParsingError::UnknownInstruction {
                location,
                line,
                invalid_instruction,
            } => {
                print_exit_on_error(
                    &path,
                    true,
                    location,
                    &line,
                    &format!("Unknown instruction: {invalid_instruction}"),
                );
            }
            FileParsingError::UnknownConst {
                location,
                line,
                invalid_const,
            } => {
                print_exit_on_error(
                    &path,
                    true,
                    location,
                    &line,
                    &format!("Invalid const: {invalid_const}"),
                );
            }
            FileParsingError::InvalidArg {
                location,
                line,
                invalid_arg,
            } => print_exit_on_error(
                &path,
                true,
                location,
                &line,
                &format!("Invalid arg: {invalid_arg}"),
            ),
        })
        .ok()
}

fn handle_vm_result(path: &OsStr, vm: &mut OpVM) -> bool {
    vm.run().map_err(|err| { match err {
        ExecutionError::TypeError {
            location,
            instruction,
            op_num,
            operand,
           expected_type,
        } => {
            print_exit_on_error(
                &path,
                false,
                location,
                &instruction.to_string(),
                &format!(
                    "Invalid type: expected {:?} for operand number {op_num}, found value {:?}",
                    expected_type, operand,
                ),
            );
        }
        ExecutionError::MissingOperand {
            location,
            instruction,
            ops_needed,
            ops_found,
        } => {
            print_exit_on_error(
                &path,
                false,
                location,
                &instruction.to_string(),
                &format!(
                    "Missing operand: expected {} operand(s) in stack, found only {}",
                    ops_needed, ops_found
                ),
            );
        }
        ExecutionError::RegOutOfIndex {
            location,
            instruction,
            reg_index,
            stack_len,
        } => {
            print_exit_on_error(&path, false, location, &instruction.to_string(), &format!("Invalid register access: targeting register {reg_index}, but there are only {stack_len} registers in stack"));
        }}; false
    }).is_ok()
}

/// Handles execution of a text assembly file
fn main_exec(path: OsString) -> ExitCode {
    // Parses the file and handle errors
    let Some(instructions) = handle_parse_result(&path) else {
        return ExitCode::FAILURE;
    };

    // Creates the vm
    let mut vm = OpVM::new(instructions);

    // Run the vm and handles errors
    if handle_vm_result(&path, &mut vm) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The main function, we match the command (only one possibility for now)
fn main() -> ExitCode {
    let args = Cli::parse();
    match args.command {
        Command::Exec { path } => main_exec(path),
    }
}
