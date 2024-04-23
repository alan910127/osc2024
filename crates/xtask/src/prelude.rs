use crate::tasks;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Task(#[from] tasks::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
