use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use log::info;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct DeleteTxnsTask {
    ids: Vec<uuid::Uuid>,

    #[serde(default)] // Default for bool is false
    sort: bool,
}

impl Task for DeleteTxnsTask {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError> {
        match cmd {
            Command::DeleteTxns(delete_txns_cmd) => Ok(Self {
                ids: delete_txns_cmd.ids,
                sort: delete_txns_cmd.sort,
            }),
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        info!("Deleting txns with ids {:?}", self.ids);
        let txns = execution_context.get_txns_mutable();
        txns.delete(&self.ids);
        if self.sort {
            txns.sort();
        }
        info!("Done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BudgetCfg, MonthlyGoals};
    use crate::cli::DeleteTxnsCmd;
    use crate::txn_ops::{Category, Txn, Txns};
    use chrono::{Days, NaiveDate};
    use uuid::Uuid;

    #[test]
    fn test_default_delete_txns() {
        let uuid = uuid::Uuid::new_v4();
        let yml = format!("ids:\n  - {}", uuid);
        let delete_txns_task: DeleteTxnsTask = serde_yaml::from_str(&yml).unwrap();

        assert_eq!(delete_txns_task.ids, vec![uuid]);
        assert_eq!(delete_txns_task.sort, false);
    }

    #[test]
    fn test_from_cmd_valid_delete_txns() {
        let uuid = Uuid::new_v4();
        let delete_txns_cmd = DeleteTxnsCmd {
            ids: vec![uuid],
            sort: false,
        };
        let cmd = Command::DeleteTxns(delete_txns_cmd);
        let delete_txns_task = DeleteTxnsTask::from_cmd(cmd).unwrap();

        assert_eq!(delete_txns_task.ids, vec![uuid]);
        assert_eq!(delete_txns_task.sort, false);
    }

    #[test]
    fn test_from_cmd_invalid_delete_txns() {
        assert!(
            DeleteTxnsTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_run_delete_txns_no_sort() {
        let mut test_run_variables = TestRunVariables::new();

        let execution_context = &mut test_run_variables.execution_context;
        let mut delete_txns_task = test_run_variables.delete_txns_task; // Take ownership here. .run() consumes the task
        delete_txns_task.sort = false; // Set to false just in case

        delete_txns_task.run(execution_context).unwrap();

        let updated_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(updated_txns.len(), 2);
        assert!(!updated_txns.contains(&test_run_variables.delete_txn));
        assert_eq!(updated_txns[0], test_run_variables.oldest_txn);
        assert_eq!(updated_txns[1], test_run_variables.keep_txn);
    }

    #[test]
    fn test_run_delete_txns_sort() {
        let mut test_run_variables = TestRunVariables::new();

        let execution_context = &mut test_run_variables.execution_context;
        let mut delete_txns_task = test_run_variables.delete_txns_task; // Take owndership her. .run() consumes the task.
        delete_txns_task.sort = true;

        delete_txns_task.run(execution_context).unwrap();

        let updated_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(updated_txns.len(), 2);
        assert!(!updated_txns.contains(&test_run_variables.delete_txn));
        assert_eq!(updated_txns[0], test_run_variables.keep_txn);
        assert_eq!(updated_txns[1], test_run_variables.oldest_txn);
    }

    struct TestRunVariables {
        pub execution_context: ExecutionContext,
        pub delete_txns_task: DeleteTxnsTask,
        pub keep_txn: Txn,
        pub delete_txn: Txn,
        pub oldest_txn: Txn,
    }

    impl TestRunVariables {
        pub fn new() -> Self {
            let date = NaiveDate::from_ymd_opt(2024, 3, 12).unwrap();
            let desc = "desc";

            let keep_txn = Txn {
                id: Uuid::new_v4(),
                date: date,
                description: desc.to_string(),
                amount: 123.34,
                category: Category::Wants,
            };

            let delete_txn = Txn {
                id: Uuid::new_v4(),
                description: desc.to_string(),
                ..keep_txn
            };

            let oldest_txn = Txn {
                id: Uuid::new_v4(),
                description: desc.to_string(),
                date: date - Days::new(1),
                ..keep_txn
            };

            let txns = Txns::new(
                Vec::from([oldest_txn.clone(), keep_txn.clone(), delete_txn.clone()]), // Vector init order here counts because of sort. If change order of vector init tests might fail.
                Vec::from([String::new()]),
            );
            let monthly_goals = MonthlyGoals::new(23.45, 67.89, 12.45).unwrap();
            let budget_cfg = BudgetCfg::new(monthly_goals, 123.45).unwrap();

            let execution_context = ExecutionContext::new(txns, budget_cfg, None);

            let delete_txns_task = DeleteTxnsTask {
                ids: Vec::from([delete_txn.id]),
                sort: false,
            };

            Self {
                execution_context,
                delete_txns_task,
                keep_txn,
                delete_txn,
                oldest_txn,
            }
        }
    }
}
