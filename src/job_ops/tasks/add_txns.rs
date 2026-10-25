use crate::cli::Command;
use crate::date_ops;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use crate::txn_ops::Category;
use log::info;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AddTxnsTask {
    description: String,
    amount: f64,

    #[serde(default = "date_ops::today")] // Default to today for the date
    date: chrono::NaiveDate,

    #[serde_as(as = "DisplayFromStr")] // Use from_str to deserialize the struct
    category: Category,

    #[serde(default)] // Default for bool is false
    sort: bool,
}

impl Task for AddTxnsTask {
    fn from_cmd(cmd: Command) -> Result<AddTxnsTask, TaskError> {
        match cmd {
            Command::AddTxn(add_txn_cmd) => Ok(AddTxnsTask {
                description: add_txn_cmd.description,
                amount: add_txn_cmd.amount,
                date: add_txn_cmd.date,
                category: add_txn_cmd.category,
                sort: add_txn_cmd.sort,
            }),
            _ => Err(TaskError::FromCmdError(String::from(cmd))),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        info!(
            "Adding txn with date: {}, description: {}, amount: {}, category: {}",
            &self.date, &self.description, &self.amount, &self.category
        );
        let txns = execution_context.get_txns_mutable();
        txns.add(self.date, self.description, self.amount, self.category);
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
    use crate::cli::{AddTxnCmd, CompareToMonthlyGoalsCmd};
    use crate::execution_context::ExecutionContext;
    use crate::txn_ops::{Txn, Txns};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_defaults_add_txns() {
        let yml = r#"
        description: test
        amount: 101.23
        category: wants
        "#;

        let add_txns_task: AddTxnsTask = serde_yaml::from_str(yml).unwrap();
        assert_eq!(add_txns_task.description.as_str(), "test");
        assert_eq!(add_txns_task.amount, 101.23);
        assert_eq!(add_txns_task.category, Category::Wants);
        assert_eq!(add_txns_task.sort, false);
        assert_eq!(add_txns_task.date, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_bad_category_add_txns() {
        let yml = r#"
        description: test
        amount: 101.23
        category: bad
        "#;
        assert!(serde_yaml::from_str::<AddTxnsTask>(yml).is_err())
    }

    #[test]
    fn test_category_case_sensitivity_add_txns() {
        let yml = r#"
        description: test
        amount: 101.23
        category: NeEdS
        "#;
        assert_eq!(
            serde_yaml::from_str::<AddTxnsTask>(yml).unwrap().category,
            Category::Needs
        )
    }

    #[test]
    fn test_from_cmd_valid_add_txns() {
        let add_txn_cmd = get_test_add_txn_cmd();
        let cmd = Command::AddTxn(add_txn_cmd.clone());
        let task = AddTxnsTask::from_cmd(cmd).unwrap();

        assert_eq!(task.date, add_txn_cmd.date);
        assert_eq!(task.description, add_txn_cmd.description);
        assert_eq!(task.amount, add_txn_cmd.amount);
        assert_eq!(task.category, add_txn_cmd.category);
        assert_eq!(task.sort, add_txn_cmd.sort);
    }

    #[test]
    fn test_from_cmd_invalid_add_txns() {
        let invalid_cmd = Command::CompareToMonthlyGoals(CompareToMonthlyGoalsCmd {
            output_path: Some("some_path".into()),
        });

        let task = AddTxnsTask::from_cmd(invalid_cmd);
        assert!(
            task.unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command comparetomonthlygoals not valid for task")
        )
    }

    #[test]
    fn test_run_sort_add_txns() {
        let mut test_run_variables = TestRunVariables::new();

        // Take ownership here - .run() will consume the add_txns_task
        let add_txns_task = test_run_variables.add_txns_task;

        let execution_context = &mut test_run_variables.execution_context;
        add_txns_task.run(execution_context).unwrap();

        let found_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(found_txns.len(), 2);

        assert_eq!(found_txns[0].date, test_run_variables.newer_date);

        let add_txn_cmd = &test_run_variables.add_txn_cmd;
        let added_txn = &found_txns[1];
        assert_eq!(added_txn.description, add_txn_cmd.description);
        assert_eq!(added_txn.amount, add_txn_cmd.amount);
        assert_eq!(added_txn.date, add_txn_cmd.date);
        assert_eq!(added_txn.category, add_txn_cmd.category);
    }

    #[test]
    fn test_run_no_sort_add_txns() {
        let mut test_run_variables = TestRunVariables::new();
        let add_txn_cmd = &mut test_run_variables.add_txn_cmd; // Mutate so can switch up the sort below
        add_txn_cmd.sort = false;

        // Take ownership here - .run() will consume the add_txns_task
        let add_txns_task = test_run_variables.add_txns_task;

        let execution_context = &mut test_run_variables.execution_context;
        add_txns_task.run(execution_context).unwrap();

        let found_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(found_txns.len(), 2);

        assert_eq!(found_txns[0].date, test_run_variables.newer_date);

        let add_txn_cmd = &test_run_variables.add_txn_cmd;
        let added_txn = &found_txns[1];
        assert_eq!(added_txn.description, add_txn_cmd.description);
        assert_eq!(added_txn.amount, add_txn_cmd.amount);
        assert_eq!(added_txn.date, add_txn_cmd.date);
        assert_eq!(added_txn.category, add_txn_cmd.category);
    }

    struct TestRunVariables {
        pub add_txn_cmd: AddTxnCmd,
        pub newer_date: NaiveDate,
        pub execution_context: ExecutionContext,
        pub add_txns_task: AddTxnsTask,
    }

    impl TestRunVariables {
        pub fn new() -> Self {
            let monthly_goals = MonthlyGoals::new(100.56, 56.56, 459.23).unwrap();
            let budget_cfg = BudgetCfg::new(monthly_goals, 100.23).unwrap();
            let add_txn_cmd = get_test_add_txn_cmd();

            let newer_date = add_txn_cmd.date + chrono::Days::new(1);
            let newer_txn = Txn {
                id: Uuid::new_v4(),
                date: newer_date,
                description: "Newer Description".to_string(),
                amount: 89.23,
                category: Category::Needs,
            };

            let txns = Txns::new(vec![newer_txn], vec!["header".to_string()]);

            let execution_context = ExecutionContext::new(txns, budget_cfg.clone(), None);

            let add_txns_task = AddTxnsTask {
                description: add_txn_cmd.description.clone(),
                amount: add_txn_cmd.amount,
                date: add_txn_cmd.date,
                category: add_txn_cmd.category,
                sort: false,
            };

            Self {
                add_txn_cmd,
                newer_date,
                execution_context,
                add_txns_task,
            }
        }
    }

    fn get_test_add_txn_cmd() -> AddTxnCmd {
        AddTxnCmd {
            date: NaiveDate::from_ymd_opt(2024, 3, 12).unwrap(),
            description: String::from("Description"),
            amount: 101.23,
            category: Category::Deposit,
            sort: true,
        }
    }
}
