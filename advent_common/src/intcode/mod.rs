mod errors;
mod memory;
mod opcodes;
mod parameters;
pub mod ports;
pub mod program;
pub mod status;
pub mod vm;

use crate::intcode::parameters::Parameter;
pub use ports::{Port, VecPort};
pub use program::Program;
pub use status::Status;
pub use vm::VM;

pub trait ReadInt {
    fn read_int(&mut self) -> anyhow::Result<i64>;
}

pub trait WriteInt {
    fn write_int(&mut self, i: i64) -> anyhow::Result<()>;
}

pub trait PortType: ReadInt + WriteInt {}

impl<T: ReadInt + WriteInt> PortType for T {}

pub trait Runable {
    fn run_with_input(&mut self, input: i64) -> Status;
    fn run(&mut self) -> Status;
}

pub trait Runner: Iterator<Item = anyhow::Result<i64>> {
    type VM: VMType;
    type Port: PortType;

    fn run(vm: Self::VM, port: Self::Port) -> Self;

    fn port(&self) -> &Self::Port;
    fn port_mut(&mut self) -> &mut Self::Port;

    fn vm(&self) -> &Self::VM;
    fn vm_mut(&mut self) -> &mut Self::VM;
}

pub struct Executor<V: VMType, P: PortType> {
    v: V,
    p: P,
}

impl<V: VMType, P: PortType> Executor<V, P> {
    fn on_exit(&mut self, status: Status) -> Option<anyhow::Result<i64>> {
        match status {
            Status::Exited(e) => e.err().map(Err),
            Status::HasOutput(out) => {
                if let Some(e) = self.p.write_int(out).err() {
                    Some(Err(e))
                } else {
                    Some(Ok(out))
                }
            }
            Status::RequiresInput => match self.p.read_int() {
                Ok(input) => {
                    let status = self.v.run_with_input(input);
                    self.on_exit(status)
                }
                err => Some(err),
            },
        }
    }
}

impl<V: VMType, P: PortType> Iterator for Executor<V, P> {
    type Item = anyhow::Result<i64>;

    fn next(&mut self) -> Option<Self::Item> {
        let status = self.v.run();
        self.on_exit(status)
    }
}

impl<V: VMType, P: PortType> Runner for Executor<V, P> {
    type VM = V;
    type Port = P;

    fn run(vm: Self::VM, port: Self::Port) -> Self {
        Self { v: vm, p: port }
    }

    fn port(&self) -> &Self::Port {
        &self.p
    }
    fn port_mut(&mut self) -> &mut Self::Port {
        &mut self.p
    }

    fn vm(&self) -> &Self::VM {
        &self.v
    }
    fn vm_mut(&mut self) -> &mut Self::VM {
        &mut self.v
    }
}

pub trait Memory {
    fn load(&self, idx: usize) -> Option<&i64>;
    fn load_mut(&mut self, idx: usize) -> Option<&mut i64>;
}

pub trait VMType: Runable + Memory {
    fn input_to(&mut self, location: Parameter);
    fn output(&mut self, output: i64);
    fn load_program(&mut self, program: &Program) -> anyhow::Result<()>;
    fn ip(&self) -> usize;
    fn offset_relative_base(&mut self, base: i64);
    fn load_rel(&self, idx: i64) -> Option<&i64>;
    fn load_rel_mut(&mut self, idx: i64) -> Option<&mut i64>;
    fn advance(&mut self, amount: usize) -> &mut Self;
    fn jump_to(&mut self, to: usize) -> &mut Self;
    fn exit(&mut self);
}
