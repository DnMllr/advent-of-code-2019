mod errors;
mod opcodes;
mod parameters;
pub mod ports;
pub mod vm;

pub use ports::{Port, VecPort};
pub use vm::VM;

pub trait ReadInt {
    fn read_int(&mut self) -> anyhow::Result<i32>;
}

pub trait WriteInt {
    fn write_int(&mut self, i: i32) -> anyhow::Result<()>;
}

pub trait PortType: ReadInt + WriteInt {}

impl<T: ReadInt + WriteInt> PortType for T {}

pub trait VMType {
    type Port: PortType;
    fn reset(&mut self) -> &mut Self;
    fn ip(&self) -> usize;
    fn load(&self, idx: usize) -> Option<&i32>;
    fn load_mut(&mut self, idx: i32) -> Option<&mut i32>;
    fn advance(&mut self, amount: usize) -> &mut Self;
    fn jump_to(&mut self, to: usize) -> &mut Self;
    fn eval(&mut self) -> anyhow::Result<i32>;
    fn port(&mut self) -> &mut Self::Port;
}
