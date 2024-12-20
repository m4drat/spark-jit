use std::collections::HashMap;

use sha2::Digest;
use spark_jit::arch::x86::Operand;
use spark_jit::arch::x86::Operand::{Imm64, MemDisp, Reg};
use spark_jit::arch::x86::Reg64;
use spark_jit::arch::x86::Reg64::*;
use spark_jit::executable::Executable;
use spark_jit::X86Asm;

use crate::rpn_converter::RPNExpr;

pub enum CompilerError {
    UnsupportedOp(crate::tokenizer::Op),
    UnknownOp(crate::tokenizer::Op),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompilerError::UnsupportedOp(op) => write!(f, "Unsupported operation: {:?}", op),
            CompilerError::UnknownOp(op) => write!(f, "Unknown operation: {:?}", op),
        }
    }
}

const ARG1: Reg64 = R8;
const ARG2: Reg64 = R9;
const VARS_BASE: Reg64 = R13;
const EVAL_STACK: Reg64 = R14;
const SCRATCH_REG: Reg64 = R15;

/// X86-64 calling convention: RDI, RSI, RDX, RCX, R8, R9, ... <stack>
const SYSTEMV_CALLING_CONV: [Reg64; 6] = [Rdi, Rsi, Rdx, Rcx, R8, R9];

/// A JIT compiler for RPN expressions.
///
/// Given an RPN expression, this compiler generates machine code that evaluates the expression.
#[derive(Default)]
pub struct Compiler {
    /// Mapping of variable names to their offsets in the variables area.
    variables_map: HashMap<String, usize>,
    /// Hasher for the integrity of the generated code.
    integrity_hasher: sha2::Sha256,
}

/// Macro that updates the integrity hash of the code generated within a block.
macro_rules! with_integrity {
    ($self:ident, $codegen:ident, $block:block) => {
        let start = $codegen.code().len();
        $block;
        let end = $codegen.code().len();
        $self.update_integrity(&$codegen.code()[start..end]);
    };
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            variables_map: HashMap::new(),
            integrity_hasher: sha2::Sha256::new(),
        }
    }

    /// Update the integrity hash with the given code bytes.
    ///
    /// # Arguments
    ///
    /// * `code` - The code bytes to hash.
    fn update_integrity(&mut self, code: &[u8]) {
        self.integrity_hasher.update(code);
    }

    /// Push a value onto the evaluation stack.
    ///
    /// # Arguments
    ///
    /// * `codegen` - The code generator.
    /// * `op` - The operand to push onto the stack.
    fn push_eval_stack(&mut self, codegen: &mut X86Asm, op: Operand) {
        with_integrity!(self, codegen, {
            match op {
                // Support 64-bit immediate values
                Imm64(_) => {
                    codegen.mov(Reg(SCRATCH_REG), op);
                    codegen.mov(MemDisp(EVAL_STACK, 0), Reg(SCRATCH_REG));
                }
                _ => {
                    codegen.mov(MemDisp(EVAL_STACK, 0), op);
                }
            }

            codegen.add(Reg(EVAL_STACK), Imm64(8));
        });
    }

    /// Pop the top of the evaluation stack into the specified register.
    ///
    /// # Arguments
    ///
    /// * `codegen` - The code generator.
    /// * `reg` - The register to pop the value into.
    fn pop_eval_stack(&mut self, codegen: &mut X86Asm, reg: Reg64) {
        with_integrity!(self, codegen, {
            codegen.sub(Reg(EVAL_STACK), Imm64(8));
            codegen.mov(Reg(reg), MemDisp(EVAL_STACK, 0));
        });
    }

    /// Compile the prologue of the generated code (save preserved registers).
    fn compile_prologue(&mut self, codegen: &mut X86Asm) {
        // Save registers
        with_integrity!(self, codegen, {
            codegen.push(Reg(R12));
            codegen.push(Reg(R13));
            codegen.push(Reg(R14));
            codegen.push(Reg(R15));
            codegen.push(Reg(Rbx));
            codegen.push(Reg(Rbp));
            codegen.push(Reg(Rdi));
            codegen.push(Reg(Rsi));
        });
    }

    /// Compile the epilogue of the generated code (restore preserved registers).
    fn compile_epilogue(&mut self, codegen: &mut X86Asm) {
        // Restore registers
        with_integrity!(self, codegen, {
            codegen.pop(Reg(Rsi));
            codegen.pop(Reg(Rdi));
            codegen.pop(Reg(Rbp));
            codegen.pop(Reg(Rbx));
            codegen.pop(Reg(R15));
            codegen.pop(Reg(R14));
            codegen.pop(Reg(R13));
            codegen.pop(Reg(R12));
        });
    }

    /// Compile a call to a native function. The function must be ABI-compatible
    /// with the x86-64 calling convention. The result is pushed onto the evaluation stack.
    ///
    /// # Arguments
    ///
    /// * `codegen` - The code generator.
    /// * `func` - Raw pointer to an ABI-compatible function.
    /// * `args` - The arguments to pass to the function.
    ///
    fn compile_native_call(&mut self, codegen: &mut X86Asm, func: usize, args: &[Operand]) {
        if args.len() > SYSTEMV_CALLING_CONV.len() {
            unimplemented!("Too many arguments for a native call!");
        }

        with_integrity!(self, codegen, {
            // Move the arguments into the correct registers
            for (i, arg) in args.iter().enumerate() {
                codegen.mov(Reg(SYSTEMV_CALLING_CONV[i]), *arg);
            }
        });

        // Make sure not to hash the function pointer!
        codegen.mov(Reg(Rax), Imm64(func as i64));

        with_integrity!(self, codegen, {
            codegen.call(Reg(Rax));
        });

        self.push_eval_stack(codegen, Reg(Rax));
    }

    /// Compile an RPN expression into machine code.
    ///
    /// # Arguments
    ///
    /// * `rpn` - An RPN expression to compile.
    ///
    /// # Returns
    ///
    /// The compiled executable.
    ///
    pub fn compile(&mut self, rpn: &RPNExpr) -> Result<Executable, CompilerError> {
        use crate::tokenizer::Op::*;
        use crate::tokenizer::Token::*;

        // We have the base address of our eval stack in RDI
        let mut codegen = X86Asm::new();

        self.compile_prologue(&mut codegen);

        // Load arguments into registers
        with_integrity!(self, codegen, {
            codegen.mov(Reg(EVAL_STACK), Reg(Rdi));
            codegen.mov(Reg(VARS_BASE), Reg(Rsi));
        });

        for token in rpn.iter() {
            match token {
                Variable(name) => {
                    let len = self.variables_map.len();
                    let offset = self
                        .variables_map
                        .entry(name.clone())
                        .or_insert_with(|| len);

                    with_integrity!(self, codegen, {
                        codegen.mov(Reg(SCRATCH_REG), Reg(VARS_BASE));
                        codegen.add(Reg(SCRATCH_REG), Imm64(*offset as i64 * 8));
                        codegen.mov(Reg(SCRATCH_REG), MemDisp(SCRATCH_REG, 0));
                    });
                    self.push_eval_stack(&mut codegen, Reg(SCRATCH_REG));
                }
                Number(n) => {
                    self.push_eval_stack(&mut codegen, Imm64(*n));
                }
                BinaryOp(op) => {
                    self.pop_eval_stack(&mut codegen, ARG1);
                    self.pop_eval_stack(&mut codegen, ARG2);

                    match op {
                        Plus => {
                            with_integrity!(self, codegen, {
                                codegen.add(Reg(ARG1), Reg(ARG2));
                            });
                            self.push_eval_stack(&mut codegen, Reg(ARG1));
                        }
                        Minus => {
                            with_integrity!(self, codegen, {
                                codegen.sub(Reg(ARG2), Reg(ARG1));
                            });
                            self.push_eval_stack(&mut codegen, Reg(ARG2));
                        }
                        Mult => {
                            with_integrity!(self, codegen, {
                                codegen.mov(Reg(Rax), Reg(ARG1));
                                codegen.imul(Reg(ARG2));
                            });
                            self.push_eval_stack(&mut codegen, Reg(Rax));
                        }
                        Div => {
                            with_integrity!(self, codegen, {
                                codegen.mov(Reg(Rax), Reg(ARG2));
                                codegen.cqo();
                                codegen.idiv(Reg(ARG1));
                            });
                            self.push_eval_stack(&mut codegen, Reg(Rax));
                        }
                        Pow => self.compile_native_call(
                            &mut codegen,
                            super::builtins::pow as usize,
                            &[Reg(ARG2), Reg(ARG1)],
                        ),
                        _ => return Err(CompilerError::UnknownOp(op.clone())),
                    }
                }
                UnaryOp(op) => {
                    self.pop_eval_stack(&mut codegen, ARG1);

                    match op {
                        Plus => {
                            self.push_eval_stack(&mut codegen, Reg(ARG1));
                        }
                        Minus => {
                            with_integrity!(self, codegen, {
                                codegen.neg(Reg(ARG1));
                            });
                            self.push_eval_stack(&mut codegen, Reg(ARG1));
                        }
                        Fact => self.compile_native_call(
                            &mut codegen,
                            super::builtins::factorial as usize,
                            &[Reg(ARG1)],
                        ),
                        _ => return Err(CompilerError::UnknownOp(op.clone())),
                    }
                }
                _ => panic!("Unexpected token"),
            }
        }

        // The result is on top of the stack.
        self.pop_eval_stack(&mut codegen, Rax);

        self.compile_epilogue(&mut codegen);
        with_integrity!(self, codegen, {
            codegen.ret();
        });

        // Allocate memory for the code and copy the generated code.
        let exec = Executable::new(
            codegen.code(),
            self.integrity_hasher.clone().finalize().as_slice(),
            self.variables_map.clone(),
        );

        // println!("Generated expression code:");
        // codegen.dump_generated_code(exec.code.as_ref().unwrap().ptr() as u64);
        // println!("Integrity hash: {}", hex::encode(&exec.integrity));

        println!("Code loaded at: {:p}", exec.code.as_ref().unwrap().ptr());

        Ok(exec)
    }
}
