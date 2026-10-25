use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::{
    AddTxnsTask, CompareToMonthlyGoalsTask, DeleteTxnsTask, GetCurrentWorthTask, GetTxnsTask,
    OffsetCalcTask, Task, TaskError, UpdateBudgetCfgTask, UpdateTxnsTask,
};

#[derive(Debug)]
pub enum JobError {
    Base(String),
    DeserializationError(String),
    JobFailure(String),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JobError::Base(err_msg)
            | JobError::DeserializationError(err_msg)
            | JobError::JobFailure(err_msg) => {
                write!(f, "{err_msg}")
            }
        }
    }
}

impl std::error::Error for JobError {}

impl From<std::io::Error> for JobError {
    fn from(err: std::io::Error) -> JobError {
        JobError::Base(format!(
            "Failed to read job file contents. Error message {}",
            err
        ))
    }
}

impl From<serde_yaml::Error> for JobError {
    fn from(err: serde_yaml::Error) -> JobError {
        JobError::DeserializationError(format!(
            "Failed to derialize job file string with Serde. Error message {}",
            err
        ))
    }
}

impl From<TaskError> for JobError {
    fn from(value: TaskError) -> Self {
        Self::JobFailure(value.to_string())
    }
}

// TODO - Rename to kebab-case for job and all tasks
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "task", content = "args")]
enum JobTask {
    AddTxns(Vec<AddTxnsTask>),
    GetTxns(GetTxnsTask),
    DeleteTxns(DeleteTxnsTask),
    UpdateTxns(Vec<UpdateTxnsTask>),
    UpdateBudgetCfg(UpdateBudgetCfgTask),
    OffsetCalc(OffsetCalcTask),
    CompareToMonthlyGoals(CompareToMonthlyGoalsTask),
    GetCurrentWorth(GetCurrentWorthTask),
}

impl JobTask {
    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), JobError> {
        let job_run_result = match self {
            Self::AddTxns(tasks) => run_task_vec(tasks, execution_context),
            Self::GetTxns(task) => task.run(execution_context),
            Self::DeleteTxns(task) => task.run(execution_context),
            Self::UpdateTxns(tasks) => run_task_vec(tasks, execution_context),
            Self::UpdateBudgetCfg(task) => task.run(execution_context),
            Self::OffsetCalc(task) => task.run(execution_context),
            Self::CompareToMonthlyGoals(task) => task.run(execution_context),
            Self::GetCurrentWorth(task) => task.run(execution_context),
        };

        Ok(job_run_result?)
    }
}

fn run_task_vec<T: Task>(
    tasks: Vec<T>,
    execution_context: &mut ExecutionContext,
) -> Result<(), TaskError> {
    tasks
        .into_iter()
        .try_for_each(|task| task.run(execution_context))
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    tasks: Vec<JobTask>,
}

impl Job {
    pub fn from_job_file<P: AsRef<std::path::Path>>(path: &P) -> Result<Self, JobError> {
        let contents = std::fs::read_to_string(path)?;
        let job: Job = serde_yaml::from_str(&contents)?;
        Ok(job)
    }

    pub fn run(self, execution_context: &mut ExecutionContext) -> Result<(), JobError> {
        self.tasks
            .into_iter()
            .try_for_each(|job_task| job_task.run(execution_context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_from_job_file_job() {
        let tempfile = tempfile::NamedTempFile::with_suffix(".yml").unwrap();

        let contents = indoc! {r#"tasks:
              - task: add-txns
                args:
                  - date: 2025-02-01
                    description: coffee
                    amount: 4.30
                    category: wants
                    sort: true

              - task: compare-to-monthly-goals
                args:

              - task: update-budget-cfg
                args:
                  savings-goal: 1000.23

              - task: get-current-worth
                args:
        "#};

        std::fs::write(&tempfile, contents).unwrap();

        let job = Job::from_job_file(&tempfile).unwrap();
        assert_eq!(job.tasks.len(), 4);
    }
}
