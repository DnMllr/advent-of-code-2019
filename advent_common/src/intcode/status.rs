#[derive(Debug)]
pub enum Status {
    Exited(anyhow::Result<()>),
    HasOutput(i64),
    RequiresInput,
}
