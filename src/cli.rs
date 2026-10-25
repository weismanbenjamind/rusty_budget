use crate::date_ops::{parse_date, today};
use crate::txn_ops::Category;
use chrono::NaiveDate;
use clap::Args as ClapArgs;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "Rusty Budget")]
#[command(about = "CLI tool for tracking budget written in Rust")]
pub struct Args {
    #[arg(
        long,
        short,
        default_value = "cfg/budget_cfg.yml",
        help = "Path to configuration file for budget"
    )]
    pub cfg: String,

    #[arg(
        long,
        short,
        default_value = "db/txns.csv",
        help = "Path to csv file serving as transactions database"
    )]
    pub txn_db: String,

    #[arg(long, short, action=clap::ArgAction::SetTrue, help = "Get version of rusty_budget; will short circuit and only print version and not run any other commands")]
    pub version: bool,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

// TODO - Maybe have sort follow the same patter as the other arguments - give it it's own cmd
#[derive(Subcommand, PartialEq, Debug)]
pub enum Command {
    AddTxn(AddTxnCmd),
    GetTxns(GetTxnsCmd),
    DeleteTxns(DeleteTxnsCmd),
    UpdateBudgetCfg(UpdateBudgetCfgCmd),
    UpdateTxn(UpdateTxnCmd),
    OffsetCalc(OffsetCalcCmd),
    CompareToMonthlyGoals(CompareToMonthlyGoalsCmd),
    GetCurrentWorth(GetCurrenWorthCmd),
    JobRun(JobRunCmd),
    /// Sort transactions
    Sort,
}

impl From<Command> for String {
    fn from(command: Command) -> String {
        match command {
            Command::AddTxn(_) => String::from("AddTxn"),
            Command::GetTxns(_) => String::from("GetTxns"),
            Command::DeleteTxns(_) => String::from("DeleteTxns"),
            Command::UpdateBudgetCfg(_) => String::from("UpdateBudgetCfg"),
            Command::UpdateTxn(_) => String::from("UpdateTxn"),
            Command::OffsetCalc(_) => String::from("OffsetCalc"),
            Command::CompareToMonthlyGoals(_) => String::from("CompareToMonthlyGoals"),
            Command::GetCurrentWorth(_) => String::from("GetCurrentWorth"),
            Command::JobRun(_) => String::from("JobRun"),
            Command::Sort => String::from("Sort"),
        }
    }
}

#[derive(Debug, Error)]
pub enum CmdValidationError {
    #[error("Arguments in invalid state. {0}")]
    InvaliadState(String),
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Add new transaction")]
pub struct AddTxnCmd {
    #[arg(long, short = 't', default_value_t = today(), help = "Date of transaction")]
    pub date: NaiveDate,

    #[arg(long, short, help = "Description of transaction")]
    pub description: String,

    #[arg(long, short, help = "Transaction amount")]
    pub amount: f64,

    #[arg(long, short, value_parser = Category::from_str, help = "Transaction category")]
    pub category: Category,

    #[arg(
        long,
        short,
        action,
        help = "If the transactions should be sorted after the analysis"
    )]
    pub sort: bool,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Compare current state of monthly budget to monthly goals")]
pub struct CompareToMonthlyGoalsCmd {
    #[arg(
        long,
        short,
        help = "Optional output path for comparison; if not provided, comparison results will be printed to the console"
    )]
    pub output_path: Option<PathBuf>,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Delte transactions by id")]
pub struct DeleteTxnsCmd {
    #[arg(
        long,
        short,
        short,
        value_delimiter = ' ',
        help = "Space delimited list of ids to delete"
    )]
    pub ids: Vec<Uuid>,

    #[arg(
        short,
        long,
        action,
        help = "If the transactions should be sorted after the analysis"
    )]
    pub sort: bool,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Get current worth")]
pub struct GetCurrenWorthCmd {
    #[arg(
        long,
        short,
        help = "Optional output path for comparison; if not provided, comparison results will be printed to the console"
    )]
    pub output_path: Option<PathBuf>,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Get a subset of transactions between two dates")]
pub struct GetTxnsCmd {
    #[arg(long, short = 'n', default_value_t = today(), value_parser = parse_date, help = "How far back (inclusivly) to get transactions")]
    pub since: NaiveDate,

    #[arg(long, short, default_value_t = today(), value_parser = parse_date, help = "How far forward (inclusivly) to get transactions")]
    pub until: NaiveDate,

    #[arg(
        long,
        short,
        help = "Optional output path for comparison; if not provided, the comparison results will be printed to the console"
    )]
    pub output_path: Option<PathBuf>,

    #[arg(
        long,
        short,
        action,
        help = "If the found transactions should be sorted"
    )]
    pub sort: bool,
}

#[derive(Debug, ClapArgs, PartialEq, Clone)]
#[command(about = "Run a job (series of commands) defined by a job file")]
pub struct JobRunCmd {
    #[arg(
        long,
        short,
        default_value = "jobs/job.yml",
        help = "Path to job file."
    )]
    pub job_file_path: String,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Do the offset calculation to adjust for any transaction input errors")]
pub struct OffsetCalcCmd {
    #[arg(long, short, help = "Current balance of chase checking account")]
    pub checking_balance: f64,

    #[arg(long, short, help = "Current balance on freedom unlimited credit card")]
    pub freedom_unlimited_balance: f64,

    #[arg(long, short, help = "Current balance on discover credit card")]
    pub discover_balance: f64,

    #[arg(long, short = 'u', help = "Current balance of huntington bank account")]
    pub huntington_balance: f64,

    #[arg(long, short = 'o', help = "Current balance of SoFi bank account")]
    pub sofi_balance: f64,

    #[arg(long, short, help = "Current amount of money in my wallet")]
    pub wallet_balance: f64,

    #[arg(
        long,
        short,
        action,
        help = "If the transactions should be sorted after the analysis"
    )]
    pub sort: bool,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Update the budget configuration file for the CLI")]
pub struct UpdateBudgetCfgCmd {
    #[arg(
        long,
        short,
        help = "Optional amount the savings goal should be set to; if no value is passed, the savings goal is left as is"
    )]
    pub savings_goal: Option<f64>,

    #[arg(
        long,
        short,
        help = "Optional amount the wants goal should be set to; if no value is passed, the wants goal is left as is"
    )]
    pub wants_goal: Option<f64>,

    #[arg(
        long,
        short,
        help = "Optional amount the needs goal should be set to; if no value is passed, the needs goal is left as is"
    )]
    pub needs_goal: Option<f64>,

    #[arg(
        long,
        short = 'k',
        help = "Optional amount the weekly wants goal should be set to; if no value is passed, the weekly wants goal is left as is; Cannot be passed if --auto-update-weekly-wants-goal (-a) is passed"
    )]
    pub weekly_wants_goal: Option<f64>,

    #[arg(
        long,
        short,
        action,
        conflicts_with = "weekly_wants_goal",
        help = "Automatically adjust the weekly wants goal based on this months current wants spend and the amount of days left in the month/week; Cannot be passed if --weekly-wants-goal (-k) is passed"
    )]
    pub auto_update_weekly_wants_goal: bool,
}

#[derive(ClapArgs, PartialEq, Debug, Clone)]
#[command(about = "Update a transaction")]
pub struct UpdateTxnCmd {
    #[arg(long, short, help = "ID of transaction to update")]
    pub id: Uuid,

    #[arg(
        long,
        short = 't',
        help = "Optional date to change the transaction date to; if not passed the transaction date will remain unchanged"
    )]
    pub date: Option<NaiveDate>,

    #[arg(
        long,
        short,
        help = "Optional description to update the transaction with; if not passed the transaction description will remain unchanged"
    )]
    pub description: Option<String>,

    #[arg(
        long,
        short,
        help = "Optional amount to update the transaction with; if not passed the transaction amount will remain unchanged"
    )]
    pub amount: Option<f64>,

    #[arg(long, short, value_parser = Category::from_str, help = "Optional category to update the transaction with; if not passed the transaction amount will remain unchanged")]
    pub category: Option<Category>,

    #[arg(
        long,
        short,
        action,
        help = "If the transactions should be sorted after the analysis"
    )]
    pub sort: bool,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cli_cmd_from_str() {
        assert_eq!(String::from(Command::Sort), "Sort".to_string());

        let cmd = DeleteTxnsCmd {
            ids: vec![Uuid::new_v4()],
            sort: false,
        };

        assert_eq!(
            String::from(Command::DeleteTxns(cmd)),
            "DeleteTxns".to_string()
        );
    }
}
