use crate::VERSION;
use crate::cfg::BudgetCfg;
use crate::cli::{Args, CmdValidationError, Command, JobRunCmd};
use crate::execution_context::{ExecutionContext, ExecutionContextError};
use crate::job_ops::{Job, JobError, TaskError, TaskRunner, TaskRunnerError};
use crate::txn_ops::Txns;
use anyhow::Error as anyhowError;
use clap::Parser;
use log::info;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Run failed. Error: {0}")]
    Base(String),

    #[error("Expected cmd to be passed as arg")]
    CmdNotSet,

    #[error("Failed to read file at {0}. Error: {1}")]
    FileRead(String, String),

    #[error("Failed to write to file at {0}. Error: {1}")]
    FileWrite(String, String),

    #[error("Failed to build RuntimeConext. Error: {0}")]
    ExecutionConextBuildFailure(String),

    #[error("Failed to task run. Error: {0}")]
    TaskFailure(String),

    #[error("Failed job run. Error {0}")]
    JobFailure(String),

    #[error("{0}")]
    InvalidCmd(String),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}

impl RunError {
    fn into_file_read(path: &str, err: Box<dyn StdError>) -> Self {
        Self::FileRead(path.to_string(), err.to_string())
    }

    fn into_file_write(path: &str, err: Box<dyn StdError>) -> Self {
        Self::FileWrite(path.to_string(), err.to_string())
    }

    fn invalid_args_from_str(msg: &str) -> Self {
        Self::InvalidArgs(msg.into())
    }
}

impl From<ExecutionContextError> for RunError {
    fn from(value: ExecutionContextError) -> Self {
        Self::ExecutionConextBuildFailure(value.to_string())
    }
}

impl From<anyhowError> for RunError {
    fn from(value: anyhowError) -> Self {
        Self::Base(value.to_string())
    }
}

impl From<TaskRunnerError> for RunError {
    fn from(value: TaskRunnerError) -> Self {
        Self::TaskFailure(value.to_string())
    }
}

impl From<TaskError> for RunError {
    fn from(value: TaskError) -> Self {
        Self::TaskFailure(value.to_string())
    }
}

impl From<JobError> for RunError {
    fn from(value: JobError) -> Self {
        Self::JobFailure(value.to_string())
    }
}

impl From<CmdValidationError> for RunError {
    fn from(value: CmdValidationError) -> Self {
        Self::InvalidCmd(value.to_string())
    }
}

pub fn run() -> Result<(), RunError> {
    env_logger::init();
    info!("Parsing CLI args");
    let args = Args::parse();
    _run(args)
}

pub fn _run(args: Args) -> Result<(), RunError> {
    validate_args(&args)?;

    if args.version {
        info!("Printing version");
        run_version(None);
        info!("Finished budget update");
        return Ok(());
    }

    let cmd = args.cmd.ok_or(RunError::CmdNotSet)?;

    let txn_db_path = args.txn_db;
    let budget_cfg_path = args.cfg;

    info!("Loading txns from db at {}", txn_db_path);
    let txns = Txns::from_csv(&txn_db_path)
        .map_err(|err| RunError::into_file_read(&txn_db_path, err.into()))?;

    info!("Reading in budget cfg at {}", budget_cfg_path);
    let budget_cfg = BudgetCfg::from_yml(&budget_cfg_path)
        .map_err(|err| RunError::into_file_read(&budget_cfg_path, err.into()))?;

    let mut execution_context = ExecutionContext::new(txns, budget_cfg, None);

    match cmd {
        Command::Sort => execution_context.get_txns_mutable().sort(),
        Command::JobRun(job_run_cmd) => run_job(job_run_cmd, &mut execution_context)?,
        Command::AddTxn(_)
        | Command::GetTxns(_)
        | Command::DeleteTxns(_)
        | Command::UpdateBudgetCfg(_)
        | Command::UpdateTxn(_)
        | Command::OffsetCalc(_)
        | Command::CompareToMonthlyGoals(_)
        | Command::GetCurrentWorth(_) => run_task(cmd, &mut execution_context)?,
    };

    info!("Writing txns to db at {}", txn_db_path);
    execution_context
        .get_txns_mutable()
        .write_to_csv(&txn_db_path, false)
        .map_err(|err| RunError::into_file_write(&txn_db_path, err.into()))?;

    info!("Writing budget cfg to file at {}", budget_cfg_path);
    execution_context
        .get_budget_cfg()
        .to_yml(&budget_cfg_path)
        .map_err(|err| RunError::into_file_write(&budget_cfg_path, err.into()))?;

    info!("Finished budget update");

    Ok(())
}

fn validate_args(args: &Args) -> Result<(), RunError> {
    match !args.version && args.cmd.is_none() {
        true => Err(RunError::invalid_args_from_str(
            "Version (--version or -v) cannot be omitted while a command is omitted. \
            Either version or a command must be passed.",
        )),
        false => Ok(()),
    }
}

fn run_version(version: Option<&str>) {
    let version = version.unwrap_or(VERSION);
    println!("{version}")
}

fn run_job(
    job_run_cmd: JobRunCmd,
    execution_context: &mut ExecutionContext,
) -> Result<(), RunError> {
    Ok(Job::from_job_file(&job_run_cmd.job_file_path)?.run(execution_context)?)
}

fn run_task(cmd: Command, execution_context: &mut ExecutionContext) -> Result<(), RunError> {
    Ok(TaskRunner::new(cmd)?.run(execution_context)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::MonthlyGoals;
    use crate::cli::{
        AddTxnCmd, CompareToMonthlyGoalsCmd, DeleteTxnsCmd, GetCurrenWorthCmd, GetTxnsCmd,
        OffsetCalcCmd, UpdateBudgetCfgCmd, UpdateTxnCmd,
    };
    use crate::txn_ops::{Category, Txn};
    use chrono::NaiveDate;
    use core::panic;
    use indoc::indoc;
    use std::fs;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[test]
    fn test_run_run_version_func() {
        run_version(Some("1.2.3"));
        run_version(None);
    }

    #[test]
    fn test_run_run_job() {
        let job_file = TmpJobFile::new();
        let job_run_cmd = JobRunCmd {
            job_file_path: job_file.job_file_path_string(),
        };
        let mut execution_context = build_mock_execution_context();

        run_job(job_run_cmd, &mut execution_context).unwrap();
    }

    #[test]
    fn test_run_run_task() {
        // GetCurrentWorth doesn't require much setup so we use it to test here
        let cmd = Command::GetCurrentWorth(GetCurrenWorthCmd { output_path: None });
        let mut execution_context = build_mock_execution_context();
        run_task(cmd, &mut execution_context).unwrap();
    }

    #[test]
    fn test_validate_args() {
        let args = build_invalid_args();
        let err = validate_args(&args).unwrap_err();
        assert!(
            matches!(err, RunError::InvalidArgs(_)),
            "Expected RunError::InvalidArgs but got: {err:?}"
        )
    }

    #[test]
    fn test_run_no_version_or_cmd() {
        let args = build_invalid_args();
        let err = _run(args).unwrap_err();
        let RunError::InvalidArgs(msg) = err else {
            panic!("Expected RunError::InvalidArgs but got: {err:?}")
        };
        let msg = msg.to_lowercase();
        assert!(
            msg.contains("version (--version or -v) cannot be omitted while a command is omitted")
        )
    }

    #[test]
    fn test_run_run_version() {
        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(None, &mock_filesystem, true);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_sort() {
        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(Some(Command::Sort), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_job_cmd() {
        let job_file = TmpJobFile::new();
        let run_job_cmd = JobRunCmd {
            job_file_path: job_file.job_file_path_string(),
        };

        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(Some(Command::JobRun(run_job_cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_add_txn() {
        let add_txn_cmd = AddTxnCmd {
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            description: String::from("test"),
            amount: 100.23,
            category: Category::Needs,
            sort: false,
        };

        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(Some(Command::AddTxn(add_txn_cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_get_txns() {
        let output_path = NamedTempFile::with_suffix(".csv").unwrap();
        let really_old_date = NaiveDate::from_ymd_opt(1800, 1, 15).unwrap();
        let huge_future_date = NaiveDate::from_ymd_opt(4000, 3, 4).unwrap();
        let cmd = GetTxnsCmd {
            since: really_old_date,
            until: huge_future_date,
            output_path: Some(output_path.as_ref().to_path_buf()),
            sort: false,
        };
        let mock_filesystem = MockFilesystem::init();

        let args = build_mock_args(Some(Command::GetTxns(cmd)), &mock_filesystem, false);
        _run(args).unwrap();

        let found = fs::read_to_string(output_path).unwrap();
        assert!(found.contains(mock_filesystem.get_txn_ids().0));
        assert!(found.contains(mock_filesystem.get_txn_ids().1));
        assert!(found.contains(mock_filesystem.get_txn_ids().2));
    }

    #[test]
    fn test_run_run_delete_txns() {
        let delete_txns_cmd = DeleteTxnsCmd {
            ids: vec![Uuid::new_v4()],
            sort: false,
        };

        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(
            Some(Command::DeleteTxns(delete_txns_cmd)),
            &mock_filesystem,
            false,
        );
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_update_budget_cfg() {
        let cmd = UpdateBudgetCfgCmd {
            savings_goal: Some(100.23),
            wants_goal: None,
            needs_goal: None,
            weekly_wants_goal: None,
            auto_update_weekly_wants_goal: false,
        };

        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(Some(Command::UpdateBudgetCfg(cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_update_txn() {
        let mock_filesystem = MockFilesystem::init();

        let cmd = UpdateTxnCmd {
            id: Uuid::parse_str(mock_filesystem.get_txn_ids().0).unwrap(),
            date: None,
            description: None,
            amount: Some(23.00),
            category: None,
            sort: false,
        };

        let args = build_mock_args(Some(Command::UpdateTxn(cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_offset_calc() {
        let mock_filesystem = MockFilesystem::init();
        let cmd = OffsetCalcCmd {
            checking_balance: 500.45,
            freedom_unlimited_balance: 2.00,
            discover_balance: 4.12,
            huntington_balance: 5000.23,
            sofi_balance: 16000.45,
            wallet_balance: 100.23,
            sort: false,
        };

        let args = build_mock_args(Some(Command::OffsetCalc(cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_compare_to_monthly_goals() {
        let cmd = CompareToMonthlyGoalsCmd { output_path: None };
        let mock_filesystem = MockFilesystem::init();

        let args = build_mock_args(
            Some(Command::CompareToMonthlyGoals(cmd)),
            &mock_filesystem,
            false,
        );

        _run(args).unwrap();
    }

    #[test]
    fn test_run_run_get_current_worth() {
        let cmd = GetCurrenWorthCmd { output_path: None };

        let mock_filesystem = MockFilesystem::init();
        let args = build_mock_args(Some(Command::GetCurrentWorth(cmd)), &mock_filesystem, false);
        _run(args).unwrap();
    }

    #[test]
    fn test_run_txns_writes() {
        let target_str = "TARGET_STR_THAT_SHOULD_BE_ADDED";
        let mock_filesystem = MockFilesystem::init();
        assert!(!mock_filesystem.get_txn_db_contents().contains(target_str));

        let add_txn_cmd = AddTxnCmd {
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            description: String::from(target_str),
            amount: 100.23,
            category: Category::Needs,
            sort: false,
        };

        let args = build_mock_args(Some(Command::AddTxn(add_txn_cmd)), &mock_filesystem, false);
        _run(args).unwrap();

        assert!(
            fs::read_to_string(&mock_filesystem.txns_db_file)
                .unwrap()
                .contains(&target_str)
        );
    }

    #[test]
    fn test_run_budget_cfg_writes() {
        let target_savings = 100.23;
        let target_savings_as_str = target_savings.to_string();

        let mock_filesystem = MockFilesystem::init();
        assert!(
            !mock_filesystem
                .get_budget_cfg_contents()
                .contains(&target_savings_as_str)
        );

        let cmd = UpdateBudgetCfgCmd {
            savings_goal: Some(target_savings),
            wants_goal: None,
            needs_goal: None,
            weekly_wants_goal: None,
            auto_update_weekly_wants_goal: false,
        };

        let args = build_mock_args(Some(Command::UpdateBudgetCfg(cmd)), &mock_filesystem, false);
        _run(args).unwrap();

        assert!(
            fs::read_to_string(&mock_filesystem.budget_cfg_file)
                .unwrap()
                .contains(&target_savings_as_str)
        );
    }

    fn build_invalid_args() -> Args {
        Args {
            cfg: "budget_cfg".to_string(),
            txn_db: "txns".to_string(),
            version: false,
            cmd: None,
        }
    }

    fn build_mock_execution_context() -> ExecutionContext {
        let txn_1 = Txn {
            id: Uuid::new_v4(),
            description: "bleh".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            amount: 100.00,
            category: Category::Deposit,
        };

        let txn_2 = Txn {
            id: Uuid::new_v4(),
            description: "blen".to_string(),
            amount: 25.00,
            category: Category::Wants,
            ..txn_1
        };

        let txn_3 = Txn {
            id: Uuid::new_v4(),
            description: "blen".to_string(),
            amount: 25.00,
            category: Category::Needs,
            ..txn_1
        };

        let txns = Txns::new(vec![txn_1, txn_2, txn_3], vec![]);
        let monthly_goals = MonthlyGoals::new(100.23, 23.23, 78.23).unwrap();
        let budget_cfg = BudgetCfg::new(monthly_goals, 67.90).unwrap();

        ExecutionContext::new(txns, budget_cfg, None)
    }

    fn build_mock_args(
        cmd: Option<Command>,
        mock_filesystem: &MockFilesystem,
        version: bool,
    ) -> Args {
        Args {
            cfg: mock_filesystem.budget_cfg_string_path(),
            txn_db: mock_filesystem.txns_db_string_path(),
            version,
            cmd: cmd,
        }
    }

    struct TmpJobFile {
        job_file: NamedTempFile,
    }

    impl TmpJobFile {
        pub fn new() -> Self {
            let job_file = NamedTempFile::with_suffix(".yml").unwrap();
            let job_contents = "tasks:\n\n !sort";
            fs::write(&job_file, job_contents).unwrap();
            Self { job_file }
        }

        pub fn job_file_path_string(&self) -> String {
            as_ref_path_to_string(&self.job_file)
        }
    }

    struct MockFilesystem {
        txns_db_file: NamedTempFile,
        budget_cfg_file: NamedTempFile,
        txn_ids: (&'static str, &'static str, &'static str),
        txn_db_contents: String,
        budget_cfg_contents: String,
    }

    impl MockFilesystem {
        pub fn init() -> Self {
            let txn_id_1 = "72093d02-d849-490e-96b5-0cf04dd1df55";
            let txn_id_2 = "9cbae415-ed46-4185-80db-910acf86fedc";
            let txn_id_3 = "0e81d8a2-447a-410d-b588-1960e1672cf3";

            let txns = format!(
                "id,date,description,amount,category\n\
                {},2026-01-16,coffee,2.0,Wants\n\
                {},2026-01-16,lunch,7.32,Wants\n\
                {},2026-01-16,paycheck,250.0,Deposit\n",
                txn_id_1, txn_id_2, txn_id_3
            );

            let budget_cfg = indoc! {"
                monthly_goals:
                  savings: 1000.00
                  needs: 4514.84
                  wants: 809.28

                weekly_wants_goal: 36.79
            "};

            let txns_db_file = NamedTempFile::with_suffix(".csv").unwrap();
            let budget_cfg_file = NamedTempFile::with_suffix(".yml").unwrap();

            fs::write(&txns_db_file, &txns).unwrap();
            fs::write(&budget_cfg_file, budget_cfg).unwrap();

            Self {
                txns_db_file,
                budget_cfg_file,
                txn_ids: (txn_id_1, txn_id_2, txn_id_3),
                txn_db_contents: txns,
                budget_cfg_contents: budget_cfg.to_string(),
            }
        }

        pub fn txns_db_string_path(&self) -> String {
            as_ref_path_to_string(&self.txns_db_file)
        }

        pub fn budget_cfg_string_path(&self) -> String {
            as_ref_path_to_string(&self.budget_cfg_file)
        }

        pub fn get_txn_ids(&self) -> &(&'static str, &'static str, &'static str) {
            &self.txn_ids
        }

        pub fn get_txn_db_contents(&self) -> &str {
            &self.txn_db_contents
        }

        pub fn get_budget_cfg_contents(&self) -> &str {
            &self.budget_cfg_contents
        }
    }

    fn as_ref_path_to_string<P: AsRef<Path>>(path: P) -> String {
        path.as_ref().to_str().unwrap().into()
    }
}
