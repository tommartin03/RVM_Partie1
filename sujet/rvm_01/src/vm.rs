use crate::inner_prelude::*;
use log::{trace, debug, info, error};

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

        trace!(
            "PC={} | Current={:?} | Stack={:?} | Executing {:?}",
            self.pc.to_idx(),
            self.current,
            self.stack,
            op
        );

        match op {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Lt | Op::Le => {
                let [op1, op2] = self.take_ops()?;
                debug!("Operands: {:?}, {:?}", op1, op2);
                // MODIF: comparaison par référence (&op1, &op2) pour éviter le move de op1 et op2
                if matches!((&op1, &op2), (Value::Int(_), Value::Int(_))) {
                    let i1 = op1.to_int().unwrap();
                    let i2 = op2.to_int().unwrap();

                    let res = match op {
                        Op::Add => (i1 + i2).into(),
                        Op::Sub => (i1 - i2).into(),
                        Op::Mul => (i1 * i2).into(),
                        Op::Div => {
                            if i2 == 0 {
                                error!("Division par zéro !");
                                return Err(ContextUpdateError::TypeError {
                                    op_num: 2,
                                    operand: Value::Int(0),
                                    expected_value: ConstType::Int,
                                });
                            }
                            (i1 / i2).into()
                        }
                        Op::Lt => (i1 < i2).into(),
                        Op::Le => (i1 <= i2).into(),
                        _ => unreachable!(),
                    };
                    self.current = res;
                } else {
                    // sinon -> float
                    // MODIF: utilisation de .clone() sur op1 et op2 pour éviter le move lors des erreurs
                    let f1 = op1.to_float().ok_or(ContextUpdateError::TypeError {
                        op_num: 1,
                        operand: op1.clone(),
                        expected_value: ConstType::Float,
                    })?;
                    let f2 = op2.to_float().ok_or(ContextUpdateError::TypeError {
                        op_num: 2,
                        operand: op2.clone(),
                        expected_value: ConstType::Float,
                    })?;

                    let res = match op {
                        Op::Add => Value::Float(f1 + f2),
                        Op::Sub => Value::Float(f1 - f2),
                        Op::Mul => Value::Float(f1 * f2),
                        Op::Div => Value::Float(f1 / f2),
                        Op::Lt => (f1 < f2).into(),
                        Op::Le => (f1 <= f2).into(),
                        _ => unreachable!(),
                    };
                    self.current = res;
                }

                trace!("Result of {:?} => {:?}", op, self.current);
            }
             // MODIF: ajout de .clone() pour éviter le move de op1 et op2 lors de la création d’erreurs
            Op::And | Op::Or => {
                let [op1, op2] = self.take_ops()?;
                debug!("Operands: {:?}, {:?}", op1, op2);
                let b1 = op1.to_bool().ok_or_else(|| ContextUpdateError::TypeError {
                    op_num: 1,
                    operand: op1.clone(),
                    expected_value: ConstType::Bool,
                })?;
                let b2 = op2.to_bool().ok_or_else(|| ContextUpdateError::TypeError {
                    op_num: 2,
                    operand: op2.clone(),
                    expected_value: ConstType::Bool,
                })?;
                let res = match op {
                    Op::And => (b1 && b2).into(),
                    Op::Or => (b1 || b2).into(),
                    _ => unreachable!(),
                };
                self.current = res;
                trace!("Result of {:?} => {:?}", op, self.current);
            }
             // Modif: On récupère d'abord le booléen réel de l'opérande dans b, puis on inverse sa valeur
            Op::Not => {
                let [operand] = self.take_ops()?;
                let b = operand.to_bool().ok_or(ContextUpdateError::TypeError {
                    op_num: 1,
                    operand,
                    expected_value: ConstType::Bool,
                })?;
                self.current = Value::Bool(!b);  // <--- le NOT logique ici !
                trace!("Result of {:?} => {:?}", op, self.current);
            }

            Op::Push => {
                self.stack.push(self.current.clone());
                trace!("Pushed {:?} to stack", self.current);
            }

            Op::Pop => {
                let [_, op] = self.take_ops()?;
                self.current = op;
                trace!("Popped {:?} from stack", self.current);
            }

            // MODIF: ajout de PushReg et PopSet
            // PushReg : duplique la valeur d’un registre sur la pile
            // PopSet  : retire le sommet de pile et l’écrit dans un registre
            &Op::PushReg(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index < self.stack.len() {
                    let stack_idx = self.stack.len() - 1 - reg_index;
                    let value = self.stack[stack_idx].clone();
                    self.stack.push(value.clone());
                    trace!("PushReg {} => valeur {:?} dupliquée sur la pile", reg_index, value);
                } else {
                    error!("Invalid PushReg index: {}", reg_index);
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
            }

            &Op::PopSet(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index >= self.stack.len() - 1 {
                    error!("Invalid PopSet index: {}", reg_index);
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
                let value = self.stack.pop().ok_or(ContextUpdateError::MissingOperand {
                    ops_found: 0,
                    ops_needed: 1,
                })?;
                let stack_idx = self.stack.len() - 1 - reg_index;
                self.stack[stack_idx] = value.clone();
                self.current = value.clone();
                trace!("PopSet {} <= valeur {:?} (pop puis set)", reg_index, value);
            }

            &Op::Get(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index < self.stack.len() {
                    let stack_idx = self.stack.len() - 1 - reg_index; // <--- mise à jour à l'index correct
                    self.current = self.stack[stack_idx].clone();
                    trace!("Get register {} => {:?}", reg_index, self.current);
                } else {
                    error!("Invalid register access: {}", reg_index);
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
            }

            &Op::Set(idx) => {
                let reg_index = idx.0 as usize;
                if reg_index < self.stack.len() {
                    let stack_idx = self.stack.len() - 1 - reg_index; // <--- mise à jour à l'index correct
                    self.stack[stack_idx] = self.current.clone();
                    trace!("Set register {} <= {:?}", reg_index, self.current);
                } else {
                    error!("Invalid register access: {}", reg_index);
                    return Err(ContextUpdateError::RegOutOfIndex { reg_index });
                }
            }

            Op::Print => {
                print!("{}", &self.current.as_printable());
                info!("Print instruction => {}", self.current.as_printable());
            }

            &Op::Jump(addr) => {
                self.pc = addr;
                trace!("Jump to PC={}", self.pc.to_idx());
            }
            &Op::Call(Addr::InstructionIdx(addr)) => {
                self.call_stack.push(self.pc);
                self.pc = Addr::InstructionIdx(addr);
                trace!("Call => PC={}, Call stack={:?}", self.pc.to_idx(), self.call_stack);
            }
            &Op::Branch(addr) => {
                let op = self.current.to_bool().unwrap();
                if op {
                    self.pc = addr;
                    trace!("Branch taken to PC={}", self.pc.to_idx());
                } else {
                    trace!("Branch not taken");
                }
            }
            Op::Ret => {
                self.pc = self.call_stack.pop().unwrap();
                self.pc.increment();
                trace!("Return => PC={}", self.pc.to_idx());
            }

            Op::Const(value) => {
                self.current = value.clone();
                trace!("Load constant => {:?}", self.current);
            }
            Op::Noop => trace!("Noop"),
            Op::Halt => {
                info!("Halt instruction encountered. Stopping execution.");
                return Err(ContextUpdateError::HaltExecution);
            }
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
            // Modif: garder l'ancienne valeur du PC pour vérifier si elle a changé
            let old_pc = insn_idx;
            // MODIF: appel simplifié de execute_instruction sans référence redondante
            match self.context.execute_instruction(instruction) {
                Ok(..) => {
                    // Modif: n'incrémente le PC que si l'instruction courante ne l'a pas déjà modifié
                    if self.context.pc.to_idx() == old_pc {
                        self.context.pc.increment();
                    }
                }
                Err(ContextUpdateError::HaltExecution) => break,
                Err(err) => {
                    let location = Location {
                        line: insn_idx + 1,
                        column: 1,
                    };
                    // MODIF: simplification du match sur err avec un return Err(match err { ... })
                    return Err(match err {
                        ContextUpdateError::TypeError {
                            op_num,
                            operand,
                            expected_value,
                        } => ExecutionError::TypeError {
                            location,
                            instruction: instruction.to_owned(),
                            op_num,
                            operand,
                            expected_type: expected_value,
                        },
                        ContextUpdateError::MissingOperand {
                            ops_found,
                            ops_needed,
                        } => ExecutionError::MissingOperand {
                            location,
                            instruction: instruction.to_owned(),
                            ops_found,
                            ops_needed,
                        },
                        ContextUpdateError::RegOutOfIndex { reg_index } => ExecutionError::RegOutOfIndex {
                            location,
                            instruction: instruction.to_owned(),
                            stack_len: self.context.stack.len(),
                            reg_index,
                        },
                        ContextUpdateError::HaltExecution => unreachable!(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.context = Context::new(Addr::InstructionIdx(0))
    }
}
