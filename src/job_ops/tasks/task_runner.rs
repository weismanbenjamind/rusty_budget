use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::{
    AddTxnsTask, CompareToMonthlyGoalsTask, DeleteTxnsTask, GetCurrentWorthTask, GetTxnsTask,
    OffsetCalcTask, Task, TaskError, UpdateBudgetCfgTask, UpdateTxnsTask,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskRunnerError {
    #[error("Failed to build task for command {0}")]
    BuildFailure(String),

    #[error("Cannot build task from cmd {0}")]
    InvalidCmd(String),
}

impl From<TaskError> for TaskRunnerError {
    fn from(value: TaskError) -> Self {
        Self::BuildFailure(value.to_string())
    }
}

pub enum TaskRunner {
    AddTxn(AddTxnsTask),
    CompareToMonthlyGoals(CompareToMonthlyGoalsTask),
    DeleteTxns(DeleteTxnsTask),
    GetCurrentWorth(GetCurrentWorthTask),
    GetTxns(GetTxnsTask),
    OffsetCalc(OffsetCalcTask),
    UpdateBudgetCfg(UpdateBudgetCfgTask),
    UpdateTxn(UpdateTxnsTask),
}

impl TaskRunner {
    pub fn new(cmd: Command) -> Result<TaskRunner, TaskRunnerError> {
        match cmd {
            Command::AddTxn(_) => Ok(Self::AddTxn(AddTxnsTask::from_cmd(cmd)?)),
            Command::CompareToMonthlyGoals(_) => Ok(Self::CompareToMonthlyGoals(
                CompareToMonthlyGoalsTask::from_cmd(cmd)?,
            )),
            Command::DeleteTxns(_) => Ok(Self::DeleteTxns(DeleteTxnsTask::from_cmd(cmd)?)),
            Command::GetCurrentWorth(_) => {
                Ok(Self::GetCurrentWorth(GetCurrentWorthTask::from_cmd(cmd)?))
            }
            Command::GetTxns(_) => Ok(Self::GetTxns(GetTxnsTask::from_cmd(cmd)?)),
            Command::OffsetCalc(_) => Ok(Self::OffsetCalc(OffsetCalcTask::from_cmd(cmd)?)),
            Command::UpdateBudgetCfg(_) => {
                Ok(Self::UpdateBudgetCfg(UpdateBudgetCfgTask::from_cmd(cmd)?))
            }
            Command::UpdateTxn(_) => Ok(Self::UpdateTxn(UpdateTxnsTask::from_cmd(cmd)?)),
            Command::JobRun(_) => Err(TaskRunnerError::InvalidCmd(cmd.into())),
            Command::Sort => Err(TaskRunnerError::InvalidCmd(cmd.into())),
        }
    }

    pub fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        match self {
            Self::AddTxn(task) => task.run(execution_context),
            Self::CompareToMonthlyGoals(task) => task.run(execution_context),
            Self::DeleteTxns(task) => task.run(execution_context),
            Self::GetCurrentWorth(task) => task.run(execution_context),
            Self::GetTxns(task) => task.run(execution_context),
            Self::OffsetCalc(task) => task.run(execution_context),
            Self::UpdateBudgetCfg(task) => task.run(execution_context),
            Self::UpdateTxn(task) => task.run(execution_context),
        }
    }
}
