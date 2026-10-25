use crate::cli::Command;
use crate::date_ops;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use crate::txn_ops::{Txn, serialize_to_csv_writer, txns_to_csv};
use chrono::NaiveDate;
use csv::Writer;
use log::info;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct GetTxnsTask {
    #[serde(default = "date_ops::today")]
    since: NaiveDate,

    #[serde(default = "date_ops::today")]
    until: NaiveDate,

    output_path: Option<PathBuf>,

    #[serde(default)] // Default for bool in false
    sort: bool,
}

impl Task for GetTxnsTask {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError> {
        match cmd {
            Command::GetTxns(get_txns_cmd) => Ok(Self {
                since: get_txns_cmd.since,
                until: get_txns_cmd.until,
                output_path: get_txns_cmd.output_path,
                sort: get_txns_cmd.sort,
            }),
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        let txns = execution_context.get_txns();

        let subset = txns.get(self.since, self.until, self.sort);

        match self.output_path {
            Some(output_path) => {
                info!("Writing txns to {:.?}", output_path);
                txns_to_csv(&subset, txns.get_header(), output_path)?;
                info!("Done");
            }
            None => {
                info!("Printing subset to console.");
                print_subset(txns.get_header(), &subset)?;
                info!("Done")
            }
        }

        Ok(())
    }
}

fn print_subset(header: &Vec<String>, subset: &Vec<&Txn>) -> Result<(), TaskError> {
    let buffer = get_txn_buffer(header, subset)?;
    write_newline_to_stdout()?;
    io::stdout().write_all(&buffer)?;
    write_newline_to_stdout()?;
    Ok(())
}

fn write_newline_to_stdout() -> Result<(), io::Error> {
    io::stdout().write_all(b"\n")
}

fn get_txn_buffer(header: &Vec<String>, subset: &Vec<&Txn>) -> Result<Vec<u8>, TaskError> {
    // Internal buffer - ensures that we don't print to console early if an error occurs
    // If just use the writer buffer we might get a print then a failure and only print part of the txns
    let mut buffer = Vec::new();
    let mut writer = Writer::from_writer(&mut buffer);
    serialize_to_csv_writer(subset, header, &mut writer)?;
    drop(writer);

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::cfg::{BudgetCfg, MonthlyGoals};
    use crate::cli::GetTxnsCmd;
    use crate::txn_ops::{Category, Txn, Txns};
    use std::fs;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_defaults_get_txns_get_txns() {
        let get_txns_task: GetTxnsTask = serde_yaml::from_str("").unwrap();
        let today = chrono::Local::now().date_naive();

        assert_eq!(get_txns_task.since, today);
        assert_eq!(get_txns_task.until, today);
        assert!(get_txns_task.output_path.is_none());
        assert_eq!(get_txns_task.sort, false);
    }

    #[test]
    fn test_from_cmd_valid_get_txns() {
        let test_context = TestContext::new(&"lol", false);
        let cmd = Command::GetTxns(test_context.get_txns_cmd.clone());
        let found = GetTxnsTask::from_cmd(cmd).unwrap();
        let cmd = test_context.get_txns_cmd;
        assert_eq!(found.output_path, cmd.output_path);
        assert_eq!(found.since, cmd.since);
        assert_eq!(found.until, cmd.until);
        assert_eq!(found.sort, cmd.sort);
    }

    #[test]
    fn test_from_cmd_invalid_get_txns() {
        assert!(
            GetTxnsTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_get_txns_no_sort_get_txns() {
        let output_path = NamedTempFile::with_suffix(".csv").unwrap();
        let mut test_context = TestContext::new(&output_path, false);
        test_context
            .get_txns_task
            .run(&mut test_context.execution_context)
            .unwrap();

        let found = fs::read_to_string(output_path).unwrap();
        assert!(!found.contains(&test_context.filtered_txn.id.to_string()));

        let keep_txn_id = test_context.keep_txn.id.to_string();
        let oldest_txn_id = test_context.oldest_txn.id.to_string();

        // Note - order of structs when initialized in the TestContext matters here
        // If updating the this initialization order the tests might fail
        let keep_txn_id_idx = found.find(keep_txn_id.as_str()).unwrap();
        let oldes_txn_id_idx = found.find(oldest_txn_id.as_str()).unwrap();
        assert!(oldes_txn_id_idx < keep_txn_id_idx);
    }

    #[test]
    fn get_txns_sort() {
        let output_path = NamedTempFile::with_suffix(".csv").unwrap();
        let mut test_context = TestContext::new(&output_path, true);
        test_context
            .get_txns_task
            .run(&mut test_context.execution_context)
            .unwrap();

        let found = fs::read_to_string(output_path).unwrap();
        assert!(!found.contains(&test_context.filtered_txn.id.to_string()));

        let oldest_id = test_context.oldest_txn.id.to_string();
        let keep_id = test_context.keep_txn.id.to_string();

        // Note - order of structs when initialized in the TestContext matters here
        // If updating the this initialization order the tests might fail
        let oldest_id_idx = found.find(oldest_id.as_str()).unwrap();
        let keep_id_idx = found.find(keep_id.as_str()).unwrap();

        assert!(keep_id_idx < oldest_id_idx);
    }

    #[test]
    fn test_print_subset() {
        let header = vec!["header".to_string()];
        let txn = Txn {
            id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(1, 2, 13).unwrap(),
            description: "test".to_string(),
            amount: 34.54,
            category: Category::Wants,
        };
        print_subset(&header, &vec![&txn]).unwrap();
    }

    #[test]
    fn test_get_txn_buffer_with_txn() {
        let header: Vec<String> = vec!["id", "date", "description", "amount", "category"]
            .iter()
            .map(|&field| field.to_string())
            .collect();

        let id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(1, 2, 13).unwrap();
        let description = "test";
        let amount = 34.54;
        let category = Category::Wants;

        let txn = Txn {
            id,
            date,
            description: description.to_string(),
            amount,
            category,
        };

        let subset = vec![&txn];

        let found = get_txn_buffer(&header, &subset).unwrap();
        let found: Vec<String> = String::from_utf8(found)
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0], header.join(","));
        assert_eq!(
            found[1],
            format!("{},{},{},{},{}", id, date, description, amount, category)
        )
    }

    #[test]
    fn test_get_txn_buffer_no_txn() {
        let header: Vec<String> = vec!["id", "date", "description", "amount", "category"]
            .iter()
            .map(|&field| field.to_string())
            .collect();

        let subset: Vec<&Txn> = Vec::new();

        let found = get_txn_buffer(&header, &subset).unwrap();
        let found: Vec<String> = String::from_utf8(found)
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0], header.join(","));
    }

    #[test]
    fn test_write_newline_to_stdout() {
        write_newline_to_stdout().unwrap();
    }

    struct TestContext {
        pub get_txns_cmd: GetTxnsCmd,
        pub keep_txn: Txn,
        pub filtered_txn: Txn,
        pub oldest_txn: Txn,
        pub execution_context: ExecutionContext,
        pub get_txns_task: GetTxnsTask,
    }

    impl TestContext {
        fn new<P: AsRef<Path>>(output_path: &P, sort: bool) -> Self {
            let keep_date = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
            let filtered_date = NaiveDate::from_ymd_opt(2025, 3, 6).unwrap();
            let oldest_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

            let keep_txn = Txn {
                id: Uuid::new_v4(),
                date: keep_date,
                description: "test".to_string(),
                amount: 110.23,
                category: Category::Wants,
            };

            let filtered_txn = Txn {
                id: Uuid::new_v4(),
                date: filtered_date,
                description: "test".to_string(),
                amount: 23.45,
                category: Category::Deposit,
            };

            let oldest_txn = Txn {
                id: Uuid::new_v4(),
                date: oldest_date,
                description: "test".to_string(),
                amount: 67.98,
                category: Category::Needs,
            };

            let txns = Txns::new(
                vec![oldest_txn.clone(), keep_txn.clone(), filtered_txn.clone()],
                vec![],
            );
            let monthly_goals = MonthlyGoals::new(103.34, 23.56, 56.45).unwrap();
            let budget_cfg = BudgetCfg::new(monthly_goals, 103.23).unwrap();

            let get_txns_cmd = GetTxnsCmd {
                since: oldest_date,
                until: keep_date,
                output_path: Some(output_path.as_ref().to_path_buf()),
                sort: sort,
            };

            let execution_context = ExecutionContext::new(txns, budget_cfg, None);

            let get_txns_task = GetTxnsTask {
                since: oldest_date,
                until: keep_date,
                output_path: Some(output_path.as_ref().to_path_buf()),
                sort: sort,
            };

            return Self {
                get_txns_cmd,
                keep_txn,
                filtered_txn,
                oldest_txn,
                execution_context,
                get_txns_task,
            };
        }
    }
}
