use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::filesystem::generate_subdirs_for_output;
use crate::job_ops::tasks::task::{Task, TaskError};
use log::info;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct GetCurrentWorthTask {
    output_path: Option<PathBuf>,
}

impl Task for GetCurrentWorthTask {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError> {
        match cmd {
            Command::GetCurrentWorth(get_current_worth_cmd) => Ok(Self {
                output_path: get_current_worth_cmd.output_path,
            }),
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        let current_worth = format!("${:.2}", execution_context.get_txns().get_current_worth());

        match self.output_path {
            Some(output_path) => {
                generate_subdirs_for_output(&output_path)?;
                info!("Writing net worth to {:.?}", output_path);
                fs::write(output_path, format!("current_worth: {current_worth}\n"))?;
                info!("Done");
            }
            None => println!("\nCurrent worth: {current_worth}\n"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BudgetCfg, MonthlyGoals};
    use crate::cli::GetCurrenWorthCmd;
    use crate::txn_ops::{Category, Txn, Txns};
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[test]
    fn test_default_get_current_worth() {
        let get_current_worth_task: GetCurrentWorthTask = serde_yaml::from_str("").unwrap();
        assert!(get_current_worth_task.output_path.is_none());
    }

    #[test]
    fn test_pass_path_buf_get_current_worth() {
        let yml = "output-path: path";
        assert_eq!(
            serde_yaml::from_str::<GetCurrentWorthTask>(yml)
                .unwrap()
                .output_path,
            Some(PathBuf::from("path"))
        );
    }

    #[test]
    fn test_from_cmd_output_path_no_write_to_console_get_current_worth() {
        let output_path = PathBuf::from("output/path.txt");
        let get_current_worth_cmd = GetCurrenWorthCmd {
            output_path: Some(output_path.clone()),
        };
        let cmd = Command::GetCurrentWorth(get_current_worth_cmd);
        let task = GetCurrentWorthTask::from_cmd(cmd).unwrap();
        assert_eq!(task.output_path, Some(output_path));
    }

    #[test]
    fn test_from_cmd_output_path_none_get_current_worth() {
        let get_current_worth_cmd = GetCurrenWorthCmd { output_path: None };
        let cmd = Command::GetCurrentWorth(get_current_worth_cmd);
        let task = GetCurrentWorthTask::from_cmd(cmd).unwrap();
        assert!(task.output_path.is_none());
    }

    #[test]
    fn test_run_get_current_worth_no_console_print() {
        let mut test_context = TestContext::new();

        let temp_file = NamedTempFile::with_suffix(".yml").unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let task = GetCurrentWorthTask {
            output_path: Some(PathBuf::from(output_path)),
        };

        task.run(&mut test_context.execution_context).unwrap();
        let found = fs::read_to_string(output_path).unwrap();
        assert_eq!(
            found,
            format!(
                "current_worth: ${:.2}\n",
                test_context.deposit_amount - test_context.wants_amount - test_context.needs_amount
            )
        )
    }

    #[test]
    fn test_run_get_current_worth_console_print() {
        let mut test_context = TestContext::new();
        let task = GetCurrentWorthTask { output_path: None };
        task.run(&mut test_context.execution_context).unwrap();
    }

    struct TestContext {
        pub deposit_amount: f64,
        pub wants_amount: f64,
        pub needs_amount: f64,
        pub execution_context: ExecutionContext,
    }

    impl TestContext {
        pub fn new() -> Self {
            let deposit_amount = 100.00;
            let wants_amount = 25.00;
            let needs_amount = 25.00;

            let txn_1 = Txn {
                id: Uuid::new_v4(),
                description: "bleh".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                amount: deposit_amount,
                category: Category::Deposit,
            };

            let txn_2 = Txn {
                id: Uuid::new_v4(),
                description: "blen".to_string(),
                amount: wants_amount,
                category: Category::Wants,
                ..txn_1
            };

            let txn_3 = Txn {
                id: Uuid::new_v4(),
                description: "blen".to_string(),
                amount: needs_amount,
                category: Category::Needs,
                ..txn_1
            };

            let txns = Txns::new(vec![txn_1, txn_2, txn_3], vec![]);

            let monthly_goals = MonthlyGoals::new(100.23, 23.23, 78.23).unwrap();

            let budget_cfg = BudgetCfg::new(monthly_goals, 67.90).unwrap();

            let execution_context = ExecutionContext::new(txns, budget_cfg, None);

            Self {
                deposit_amount,
                wants_amount,
                needs_amount,
                execution_context,
            }
        }
    }
}
