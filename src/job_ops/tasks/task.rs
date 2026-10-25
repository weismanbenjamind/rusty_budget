use crate::cfg::MonthlyGoalsError;
use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::filesystem::FilesystemError;
use crate::txn_ops::TxnsError;
use csv::Error as CsvError;
use std::error::Error;
use std::io;

#[derive(Debug)]
pub enum TaskError {
    Base(String),
    FromCmdError(String),
    InvalidState(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskError::Base(err_msg) => {
                write!(f, "{err_msg}")
            }
            TaskError::FromCmdError(cmd) => {
                write!(f, "Command {cmd} not valid for task")
            }
            TaskError::InvalidState(err_msg) => {
                write!(f, "Invalid state for task: {err_msg}.")
            }
        }
    }
}

impl Error for TaskError {}

impl From<FilesystemError> for TaskError {
    fn from(value: FilesystemError) -> Self {
        TaskError::Base(value.to_string())
    }
}

impl From<io::Error> for TaskError {
    fn from(value: io::Error) -> Self {
        Self::Base(value.to_string())
    }
}

impl From<anyhow::Error> for TaskError {
    fn from(value: anyhow::Error) -> Self {
        Self::Base(value.to_string())
    }
}

impl From<MonthlyGoalsError> for TaskError {
    fn from(value: MonthlyGoalsError) -> Self {
        TaskError::Base(value.to_string())
    }
}

impl From<TxnsError> for TaskError {
    fn from(value: TxnsError) -> Self {
        TaskError::Base(value.to_string())
    }
}

impl From<CsvError> for TaskError {
    fn from(value: CsvError) -> Self {
        TaskError::Base(value.to_string())
    }
}

pub trait Task {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError>
    where
        Self: Sized;

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError>;
}
