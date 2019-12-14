use std::fmt::Write;

use anyhow::Result;

use crate::intcode::memory::Memory;
use crate::intcode::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;
use crate::intcode::parameters::Parameter;
use crate::intcode::{Program, Runable, Status, VMType, Memory as MemoryT};

enum InternalStatus {
    Running,
    Exited(Result<(), ()>),
    Outputting(i64),
    WaitingOnInputTo(Parameter),
}

pub struct VM {
    status: InternalStatus,
    memory: Memory,
    instruction_pointer: usize,
    relative_base: i64,
}

impl VM {
    pub fn new() -> Self {
        Default::default()
    }

    fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory.as_inner()[self.ip()..])
    }

    pub fn dump(&self) -> String {
        let msg = "it should be impossible to fail here, just writing a format into memory";
        let mut s = String::new();
        for (i, num) in self.memory.iter().enumerate() {
            if i > 0 {
                write!(&mut s, ",").expect(msg);
            }
            write!(&mut s, "{}", num).expect(msg);
        }
        s
    }
}

impl Runable for VM {
    fn run_with_input(&mut self, input: i64) -> Status {
        if let InternalStatus::WaitingOnInputTo(p) = self.status {
            if let Some(err) = p.read_mut(self).map(|r| *r = input).err() {
                self.status = InternalStatus::Exited(Err(()));
                return Status::Exited(Err(err));
            }
            self.status = InternalStatus::Running;
            self.run()
        } else {
            self.status = InternalStatus::Exited(Err(()));
            Status::Exited(Err(ErrorKinds::UnexpectedInputError.into()))
        }
    }

    fn run(&mut self) -> Status {
        match self.status {
            InternalStatus::WaitingOnInputTo(_) => {
                self.status = InternalStatus::Exited(Err(()));
                Status::Exited(Err(ErrorKinds::ExpectedInputError.into()))
            }
            InternalStatus::Exited(e) => {
                Status::Exited(e.map_err(|_| ErrorKinds::RanAfterErrorExitError.into()))
            }
            _ => {
                self.status = InternalStatus::Running;
                loop {
                    if let Some(e) = self.load_inst().and_then(|inst| inst.exec(self)).err() {
                        self.status = InternalStatus::Exited(Err(()));
                        return Status::Exited(Err(e));
                    }
                    match &self.status {
                        InternalStatus::Exited(e) => {
                            return Status::Exited(
                                e.map_err(|_| ErrorKinds::RanAfterErrorExitError.into()),
                            )
                        }
                        InternalStatus::Outputting(i) => return Status::HasOutput(*i),
                        InternalStatus::WaitingOnInputTo(_) => return Status::RequiresInput,
                        InternalStatus::Running => continue,
                    }
                }
            }
        }
    }
}

impl super::Memory for VM {
    fn load(&self, idx: usize) -> Option<&i64> {
        self.memory.load(idx)
    }

    fn load_mut(&mut self, idx: usize) -> Option<&mut i64> {
        self.memory.load_mut(idx)
    }
}

impl VMType for VM {
    fn input_to(&mut self, location: Parameter) {
        self.status = InternalStatus::WaitingOnInputTo(location);
    }

    fn output(&mut self, value: i64) {
        self.status = InternalStatus::Outputting(value);
    }

    fn load_program(&mut self, program: &Program) -> Result<()> {
        self.memory.zero();
        program.load_to(self.memory.as_inner_mut())?;
        self.status = InternalStatus::Running;
        self.instruction_pointer = 0;
        Ok(())
    }

    fn ip(&self) -> usize {
        self.instruction_pointer
    }

    fn offset_relative_base(&mut self, base: i64) {
        self.relative_base += base;
    }

    fn load_rel(&self, idx: i64) -> Option<&i64> {
        self.load((self.relative_base + idx) as usize)
    }

    fn load_rel_mut(&mut self, idx: i64) -> Option<&mut i64> {
        self.load_mut((self.relative_base + idx) as usize)
    }

    fn advance(&mut self, amount: usize) -> &mut Self {
        self.jump_to(amount + self.instruction_pointer)
    }

    fn jump_to(&mut self, to: usize) -> &mut Self {
        self.instruction_pointer = to;
        self
    }

    fn exit(&mut self) {
        self.status = InternalStatus::Exited(Ok(()));
    }
}

impl Default for VM {
    fn default() -> Self {
        Self {
            status: InternalStatus::Running,
            memory: Memory::new(),
            instruction_pointer: 0,
            relative_base: 0,
        }
    }
}
