use crate::inner_prelude::*;

#[derive(Debug, Clone)]
pub struct Context {
    pub pc: Addr,
    pub stack: Vec<Value>,
    pub current: Value,
    pub call_stack: Vec<Addr>,
}

impl Context {
    /// Takes N operands in the stacks and return them
    fn take_ops<const N: usize>(&mut self) -> Result<[Value; N], ContextUpdateError> {
        let mut values = std::array::from_fn(|_| Value::Int(0));

        if 0 < N {
            values[0] = self.current.take();
        }

        for i in 1..N {
            match self.stack.pop() {
                Some(value) => values[i] = value,
                None => {
                    return Err(ContextUpdateError::MissingOperand {
                        ops_found: i,
                        ops_needed: N,
                    })
                }
            }
        }

        Ok(values)
    }

    /// Executes an instruction and updates the context
    fn execute_instruction(&mut self, op: &Instruction) -> Result<(), ContextUpdateError> {
        use Instruction as Op;
        match op {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Lt | Op::Le => {
                let [op1, op2] = self.take_ops()?;
                let i1 = op1.to_int().ok_or(ContextUpdateError::TypeError {
                    op_num: 1,
                    operand: op1,
                    expected_value: ConstType::Int,
                })?;
                let i2 = op2.to_int().ok_or(ContextUpdateError::TypeError {
                    op_num: 2,
                    operand: op2,
                    expected_value: ConstType::Int,
                })?;

                let res = match op {
                    Op::Add => (i1 + i2).into(),
                    Op::Sub => (i1 - i2).into(),
                    Op::Mul => (i1 * i2).into(),
                    Op::Div => (i1 / i2).into(),
                    Op::Lt => (i1 < i2).into(),
                    Op::Le => (i1 <= i2).into(),
                    _ => unreachable!(),
                };
                self.current = res;
            }
            Op::And | Op::Or => {
                let [op1, op2] = self.take_ops()?;
                let b1 = op1.to_bool().ok_or_else(|| ContextUpdateError::TypeError {
                    op_num: 1,
                    operand: op1,
                    expected_value: ConstType::Bool,
                })?;
                let b2 = op2.to_bool().ok_or_else(|| ContextUpdateError::TypeError {
                    op_num: 2,
                    operand: op2,
                    expected_value: ConstType::Bool,
                })?;
                let res = match op {
                    Op::And => (b1 && b2).into(),
                    Op::Or => (b1 || b2).into(),
                    _ => unreachable!(),
                };
                self.current = res;
            }
            Op::Not => {
                let [operand] = self.take_ops()?;
                self.current =
                    Value::Bool(operand.to_bool().ok_or(ContextUpdateError::TypeError {
                        op_num: 1,
                        operand,
                        expected_value: ConstType::Bool,
                    })?);
            }
            Op::Push => self.stack.push(self.current.clone()),
            Op::Pop => {
                let [_, op] = self.take_ops()?;
                self.current = op;
            }
            &Op::Get(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index < self.stack.len() {
                    let stack_idx = self.stack.len() - reg_index;
                    self.current = self.stack[stack_idx].clone()
                } else {
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
            }
            &Op::Set(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index < self.stack.len() {
                    let stack_idx = self.stack.len() - idx.0 as usize;
                    self.stack[stack_idx] = self.current.clone()
                } else {
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
            }
            Op::Print => print!("{}", &self.current.as_printable()),

            &Op::Jump(addr) => self.pc = addr,
            &Op::Call(Addr::InstructionIdx(addr)) => {
                self.call_stack.push(self.pc);
                self.pc = Addr::InstructionIdx(addr + 1)
            }
            &Op::Branch(addr) => {
                let op = self.current.to_bool().unwrap();
                if op {
                    self.pc = addr
                }
            }
            Op::Ret => self.pc = self.call_stack.pop().unwrap(),
            Op::Const(value) => self.current = value.clone(),
            Op::Noop => (),
            Op::Halt => return Err(ContextUpdateError::HaltExecution),
        };
        Ok(())
    }
}

impl Context {
    fn new(pc: Addr) -> Self {
        Context {
            pc,
            stack: Vec::new(),
            current: Value::Int(0),
            call_stack: Vec::new(),
        }
    }
}

pub struct OpVM {
    code: Vec<Instruction>,
    context: Context,
}

impl OpVM {
    pub fn new(code: Vec<Instruction>) -> Self {
        Self {
            code,
            context: Context::new(Addr::InstructionIdx(0)),
        }
    }
    
    /// Main VM function, loop through instructions, updating the context, while there is no Halt instruction
    pub fn run(&mut self) -> Result<(), ExecutionError> {
        loop {
            let insn_idx = self.context.pc.to_idx();
            let instruction = &self.code[insn_idx];
            // Handles errors ...
            match self.context.execute_instruction(&instruction) {
                Ok(..) => (),
                Err(ContextUpdateError::HaltExecution) => break,
                Err(err) => {
                    let location = Location {
                        line: insn_idx + 1,
                        column: 1,
                    };
                    match err {
                        ContextUpdateError::TypeError {
                            op_num,
                            operand,
                            expected_value,
                        } => {
                            return Err(ExecutionError::TypeError {
                                location,
                                instruction: instruction.to_owned(),
                                op_num,
                                operand,
                                expected_type: expected_value,
                            })
                        }
                        ContextUpdateError::MissingOperand {
                            ops_found,
                            ops_needed,
                        } => {
                            return Err(ExecutionError::MissingOperand {
                                location,
                                instruction: instruction.to_owned(),
                                ops_found,
                                ops_needed,
                            })
                        }

                        ContextUpdateError::RegOutOfIndex { reg_index } => {
                            return Err(ExecutionError::RegOutOfIndex {
                                location,
                                instruction: instruction.to_owned(),
                                stack_len: self.context.stack.len(),
                                reg_index,
                            })
                        }
                        ContextUpdateError::HaltExecution => unreachable!(),
                    }
                }
            }
            self.context.pc.increment();
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.context = Context::new(Addr::InstructionIdx(0))
    }
}