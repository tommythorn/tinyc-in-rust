//! Compile from parsed source code to code as a list of instructions.

#![warn(clippy::all, clippy::pedantic)]

use crate::parser::Node;

/// `Insn` models the instructions of our virtual machine.
///
/// Marc Feeley uses fixed size for each instruction, with Push,
/// Fetch, Load, Jmp, Jz, and Jnz taking two slots.  We model the cast
/// of the slot values with the `Integer(_)` and `Address(_)` types
/// which of course wouldn't exist in a Real Implementation.  An
/// alternative (more type safe and more conventional) approach would
/// be to use `Fetch(usize)` etc. directly.
///
/// The targets of `Jmp`, `Jnz`, and `Jz` are absolute addresses.
/// Conventionally they would be relative addresses.
#[derive(Debug)]
pub enum Insn {
    Fetch,
    Store,
    Push,
    Pop,
    Add,
    Sub,
    Lt,
    Jz,
    Jnz,
    Jmp,
    Halt,
    Integer(isize),
    Address(usize),
}

/// Take the top-level program Node and compile it to instructions.
#[must_use]
pub fn compile(ast: Node) -> Vec<Insn> {
    let mut cg = Codegen::default();
    cg.compile(ast);
    cg.code
}

/// The Generator traverses the parsed source code and generates
/// `code` in the process.
#[derive(Default)]
struct Codegen {
    code: Vec<Insn>,
}

impl Codegen {
    #[allow(clippy::unused_self)]
    fn global(&self, v: &str) -> usize {
        match v {
            v if v.len() == 1 => v.chars().next().unwrap() as usize - 97,
            _ => panic!("{v} isn't a variable we can compile right now"),
        }
    }

    fn here(&self) -> usize {
        self.code.len()
    }

    fn hole(&mut self) -> usize {
        let p = self.here();
        self.code.push(Insn::Address(0));
        p
    }

    fn fix(&mut self, hole: usize, target: usize) {
        self.code[hole] = Insn::Address(target);
    }

    fn compile(&mut self, n: Node) {
        match n {
            Node::Add(a, b) => {
                self.compile(*a);
                self.compile(*b);
                self.code.push(Insn::Add);
            }
            Node::Sub(a, b) => {
                self.compile(*a);
                self.compile(*b);
                self.code.push(Insn::Sub);
            }
            Node::If1(test, then) => {
                self.compile(*test);
                self.code.push(Insn::Jz);
                let jz = self.hole();

                self.compile(*then);
                self.fix(jz, self.here());
            }
            Node::If2(test, then, else_) => {
                self.compile(*test);
                self.code.push(Insn::Jz);
                let jz = self.hole();

                self.compile(*then);
                self.code.push(Insn::Jmp);
                let jmp = self.hole();

                self.fix(jz, self.here());
                self.compile(*else_);

                self.fix(jmp, self.here());
            }
            Node::While(test, body) => {
                let l_restart = self.here();

                self.compile(*test);

                self.code.push(Insn::Jz);
                let jz = self.hole();

                self.compile(*body);
                self.code.push(Insn::Jmp);
                let jmp = self.hole();

                self.fix(jmp, l_restart);
                self.fix(jz, self.here());
            }
            Node::Do(body, test) => {
                let l_restart = self.here();

                self.compile(*body);
                self.compile(*test);

                self.code.push(Insn::Jnz);
                let jnz = self.hole();
                self.fix(jnz, l_restart);
            }
            Node::Prog(body) => {
                self.compile(*body);
                self.code.push(Insn::Halt);
            }
            Node::Expr(body) => {
                self.compile(*body);
                self.code.push(Insn::Pop);
            }
            Node::Set(var, expr) => {
                self.compile(*expr);
                self.code.push(Insn::Store);
                let Node::Var(v) = *var else {
                    panic!("We expected a Var, not {:?}", *var);
                };
                self.code.push(Insn::Address(self.global(&v)));
            }
            Node::Cst(val) => {
                self.code.push(Insn::Push);
                self.code.push(Insn::Integer(val));
            }
            Node::Var(v) => {
                self.code.push(Insn::Fetch);
                self.code.push(Insn::Address(self.global(&v)));
            }
            Node::Lt(a, b) => {
                self.compile(*a);
                self.compile(*b);
                self.code.push(Insn::Lt);
            }
            Node::Seq(a, b) => {
                self.compile(*a);
                self.compile(*b);
            }
            Node::Empty => {}
        }
    }
}
