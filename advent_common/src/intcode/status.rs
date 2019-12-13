#[derive(Debug)]
pub enum Status {
    Exited(anyhow::Result<()>),
    HasOutput(i32),
    RequiresInput,
}
