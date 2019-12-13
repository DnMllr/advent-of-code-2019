use std::fmt::Write;

use anyhow::Result;

use crate::intcode::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;
use crate::intcode::parameters::Parameter;
use crate::intcode::{Program, Runable, Status, VMType};

enum InternalStatus {
    Running,
    Exited,
    Outputting(i32),
    WaitingOnInputTo(Parameter),
}

pub struct VM {
    status: InternalStatus,
    memory: Vec<i32>,
    instruction_pointer: usize,
}

impl VM {
    pub fn new() -> Self {
        Default::default()
    }

    fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory[self.ip()..])
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
    fn run_with_input(&mut self, input: i32) -> Status {
        if let InternalStatus::WaitingOnInputTo(p) = self.status {
            if let Some(err) = p.read_mut(self).map(|r| *r = input).err() {
                self.status = InternalStatus::Exited;
                return Status::Exited(Err(err));
            }
            self.status = InternalStatus::Running;
            self.run()
        } else {
            self.status = InternalStatus::Exited;
            Status::Exited(Err(ErrorKinds::UnexpectedInputError.into()))
        }
    }

    fn run(&mut self) -> Status {
        match self.status {
            InternalStatus::WaitingOnInputTo(_) => {
                self.status = InternalStatus::Exited;
                Status::Exited(Err(ErrorKinds::ExpectedInputError.into()))
            }
            InternalStatus::Exited => Status::Exited(Ok(())),
            _ => {
                self.status = InternalStatus::Running;
                loop {
                    if let Some(e) = self.load_inst().and_then(|inst| inst.exec(self)).err() {
                        self.status = InternalStatus::Exited;
                        return Status::Exited(Err(e));
                    }
                    match &self.status {
                        InternalStatus::Exited => return Status::Exited(Ok(())),
                        InternalStatus::Outputting(i) => return Status::HasOutput(*i),
                        InternalStatus::WaitingOnInputTo(_) => return Status::RequiresInput,
                        InternalStatus::Running => continue,
                    }
                }
            }
        }
    }
}

impl VMType for VM {
    fn input_to(&mut self, location: Parameter) {
        self.status = InternalStatus::WaitingOnInputTo(location);
    }

    fn output(&mut self, value: i32) {
        self.status = InternalStatus::Outputting(value);
    }

    fn load_program(&mut self, program: &Program) {
        program.load_to(&mut self.memory);
        self.status = InternalStatus::Running;
        self.instruction_pointer = 0;
    }

    fn ip(&self) -> usize {
        self.instruction_pointer
    }

    fn load(&self, idx: usize) -> Option<&i32> {
        self.memory.get(idx)
    }

    fn load_mut(&mut self, idx: i32) -> Option<&mut i32> {
        self.memory.get_mut(idx as usize)
    }

    fn advance(&mut self, amount: usize) -> &mut Self {
        self.jump_to(amount + self.instruction_pointer)
    }

    fn jump_to(&mut self, to: usize) -> &mut Self {
        self.instruction_pointer = to;
        self
    }

    fn exit(&mut self) {
        self.status = InternalStatus::Exited;
    }
}

impl Default for VM {
    fn default() -> Self {
        Self {
            status: InternalStatus::Running,
            memory: Vec::with_capacity(64),
            instruction_pointer: 0,
        }
    }
}
