/* Virtual machine. */

use crate::codegen::Insn;

/// The virtual machine executes the `Insn` and holds the `code`, the
/// `pc`, the `stack`, and the `globals`.
#[derive(Default)]
pub struct VM {
    pub globals: [isize; 26],
    code: Vec<Insn>,
    pc: usize,
    stack: Vec<isize>,
    tracing: bool,
}

impl VM {
    #[must_use]
    pub fn new() -> Self {
        VM { ..VM::default() }
    }

    pub fn trace_on(&mut self) {
        self.tracing = true;
    }

    fn get_const(&mut self) -> isize {
        let Insn::Integer(n) = self.code[self.pc] else {
            panic!("Bad code, expected integer constant, got {:?}", self.code[self.pc]);
        };
        self.pc += 1;
        n
    }

    fn get_address(&mut self) -> usize {
        let Insn::Address(n) = self.code[self.pc] else {
            panic!("Bad code, expected address constant, got {:?}", self.code[self.pc]);
        };
        self.pc += 1;
        n
    }

    fn top(&mut self) -> isize {
        self.stack[self.stack.len() - 1]
    }

    /// # Panics
    /// Panics on illegal code
    pub fn run(&mut self, code: Vec<Insn>) {
        self.code = code;
        self.pc = 0;
        loop {
            let insn = &self.code[self.pc];

            if self.tracing {
                println!("{:4}: {:?}  (stack: {:?})", self.pc, insn, self.stack);
            }

            self.pc += 1;
            match *insn {
                Insn::Integer(_) | Insn::Address(_) => {
                    panic!("Can't execute middle of instructions")
                }
                Insn::Halt => {
                    break;
                }
                Insn::Fetch => {
                    let a = self.get_address();
                    self.stack.push(self.globals[a]);
                }
                Insn::Store => self.globals[self.get_address()] = self.top(),
                Insn::Push => {
                    let v = self.get_const();
                    self.stack.push(v);
                }
                Insn::Pop => {
                    self.stack.pop().unwrap();
                }
                Insn::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Insn::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                }
                Insn::Lt => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(isize::from(a < b));
                }
                Insn::Jmp => self.pc = self.get_address(),
                Insn::Jz => {
                    let n = self.get_address();
                    let v = self.stack.pop().unwrap();
                    if v == 0 {
                        self.pc = n;
                    }
                }
                Insn::Jnz => {
                    let n = self.get_address();
                    let v = self.stack.pop().unwrap();
                    if v != 0 {
                        self.pc = n;
                    }
                }
            }
        }
    }
}
