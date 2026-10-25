use crate::cli::Command;
use crate::date_ops::get_start_of_week;
use crate::execution_context::ExecutionContext;
use crate::filesystem::{FilesystemError, generate_subdirs_for_output};
use crate::job_ops::tasks::task::{Task, TaskError};
use crate::monthly_metrics::MonthlyMetrcs;
use crate::txn_ops::Txns;
use chrono::NaiveDate;
use log::info;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CompareToMonthlyGoalsTask {
    output_path: Option<PathBuf>,
}

impl Task for CompareToMonthlyGoalsTask {
    fn from_cmd(cmd: Command) -> Result<CompareToMonthlyGoalsTask, TaskError> {
        match cmd {
            Command::CompareToMonthlyGoals(compare_to_monthly_goals_cmd) => {
                Ok(CompareToMonthlyGoalsTask {
                    output_path: compare_to_monthly_goals_cmd.output_path,
                })
            }
            _ => Err(TaskError::FromCmdError(String::from(cmd))), // Can also do cmd.into() since compilor knows we need a String and From<Command> is implemented for String
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        let today = execution_context.get_today();
        info!("Using today as {today}");

        let txns = execution_context.get_txns();
        let budget_cfg = execution_context.get_budget_cfg();

        let monthly_metrics = MonthlyMetrcs::from_txns(txns, today);

        let monthly_savings = monthly_metrics.savings();
        let monthly_offsets = monthly_metrics.into_offsets(&budget_cfg.monthly_goals);

        let weekly_wants_spend_offset =
            get_weekly_wants_spend_offset(txns, today, budget_cfg.get_weekly_wants_goal());

        let output_string =
            get_output_string(&monthly_offsets, weekly_wants_spend_offset, monthly_savings);

        match self.output_path {
            Some(output_path) => {
                info!("Writing outputs to {:.?}", output_path);
                write_outputs(&output_string, &output_path)?;
                info!("Done");
            }
            None => {
                info!("Printing comparison to stdout.");
                println!("{output_string}");
                info!("Done");
            }
        }

        Ok(())
    }
}

fn get_weekly_wants_spend_offset(
    txns: &Txns,
    date: NaiveDate,
    weekly_wants_spend_goal: f64,
) -> f64 {
    let start_of_week = get_start_of_week(date);
    let curr_weekly_spend: f64 = txns.get_wants_spend(start_of_week, date);
    weekly_wants_spend_goal - curr_weekly_spend
}

fn write_outputs<P: AsRef<Path>>(output_str: &str, output_path: &P) -> Result<(), FilesystemError> {
    generate_subdirs_for_output(output_path)?;
    std::fs::write(output_path, output_str)?;

    Ok(())
}

fn get_output_string(
    monthly_offsets: &MonthlyMetrcs,
    weekly_wants_spend_offset: f64,
    monthly_savings: f64,
) -> String {
    let monthly_needs_offset_field =
        create_output_field("monthly_needs_spend", monthly_offsets.needs(), None);

    let monthly_wants_offset_field =
        create_output_field("monthly_wants_spend", monthly_offsets.wants(), None);

    let monthly_savings_offset_field = create_output_field(
        "monthly_savings",
        monthly_offsets.savings(),
        Some("(actual-goal)"),
    );

    let monthly_savings_field =
        create_output_field("monthly_savings", monthly_savings, Some("(absolute)"));

    let weekly_wants_spend_offset_field =
        create_output_field("weekly_wants_spend", weekly_wants_spend_offset, None);

    format!(
        "\n{}{}{}{}{}",
        monthly_needs_offset_field,
        monthly_wants_offset_field,
        monthly_savings_offset_field,
        monthly_savings_field,
        weekly_wants_spend_offset_field
    )
}

fn create_output_field(name: &str, amt: f64, diff_indicator: Option<&str>) -> String {
    let diff_indicator = diff_indicator.unwrap_or("(goal-actual)");
    format!("{} {}: ${:.2}\n", name, diff_indicator, amt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BudgetCfg, MonthlyGoals};
    use crate::cli::CompareToMonthlyGoalsCmd;
    use crate::txn_ops::{Category, Txn};
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[test]
    fn test_from_cmd_valid_compare_to_monthly_goals_() {
        let output_path = "some_path";
        let compare_to_monthly_goals_cmd = CompareToMonthlyGoalsCmd {
            output_path: Some(output_path.into()),
        };
        let cmd = Command::CompareToMonthlyGoals(compare_to_monthly_goals_cmd);
        let compare_to_monthly_goals_task = CompareToMonthlyGoalsTask::from_cmd(cmd).unwrap();
        match compare_to_monthly_goals_task.output_path {
            Some(output_path_from_cmd) => {
                assert_eq!(output_path_from_cmd, PathBuf::from(output_path))
            }
            None => panic!("Output path not set properly. Expected {output_path} found None"),
        }
    }

    #[test]
    fn test_from_cmd_invalid_compare_to_monthly_goals_() {
        assert!(
            CompareToMonthlyGoalsTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_get_weekly_wants_spend_offset_compare_to_monthly_goals() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
        let amount_1 = 100.23;

        let txn_1 = Txn {
            id: Uuid::new_v4(),
            date,
            description: "test".to_string(),
            amount: amount_1,
            category: Category::Wants,
        };

        let txn_2 = Txn {
            description: txn_1.description.clone(),
            category: Category::Needs,
            ..txn_1
        };

        let txn_3 = Txn {
            description: txn_1.description.clone(),
            category: Category::Deposit,
            ..txn_1
        };

        let amount_2 = 12.34;

        let txn_4 = Txn {
            description: txn_1.description.clone(),
            amount: amount_2,
            category: Category::Wants,
            ..txn_1
        };

        let txns = Txns::new(vec![txn_1, txn_2, txn_3, txn_4], vec![]);

        let weekly_wants_goal = 150.34;

        let found = get_weekly_wants_spend_offset(&txns, date, weekly_wants_goal);
        let expected = weekly_wants_goal - amount_1 - amount_2;
        assert_eq!(found, expected);
    }

    #[test]
    fn test_create_ouptput_field_no_diff_indicator_compare_to_monthly_goals() {
        let name = "weekly_wants_spend";
        let amt = 100.56;
        let found = create_output_field(name, amt, None);
        let expected = format!("{} {}: ${:.2}\n", name, "(goal-actual)", amt);
        assert_eq!(found, expected);
    }

    #[test]
    fn test_create_ouptput_field_diff_indicator_compare_to_monthly_goals() {
        let name = "weekly_wants_spend";
        let amt = 100.56;
        let diff_indicator = Some("bleh");
        let found = create_output_field(name, amt, diff_indicator);
        let expected = format!("{} {}: ${:.2}\n", name, diff_indicator.unwrap(), amt);
        assert_eq!(found, expected);
    }

    #[test]
    fn test_write_outputs_compare_to_monthly_goals() {
        let output_file_path = NamedTempFile::with_suffix(".yml").unwrap();
        let output_str = "some: stuff\n";

        write_outputs(output_str, &output_file_path).unwrap();

        let output_contents = std::fs::read_to_string(output_file_path).unwrap();
        assert_eq!(&output_contents, output_str);
    }

    #[test]
    fn test_run_compare_to_monthly_goals_write_to_file() {
        let tmp_file = tempfile::NamedTempFile::with_suffix(".yml").unwrap();
        let output_path = tmp_file.path().to_str().unwrap();

        let compare_to_monthly_goals_task = CompareToMonthlyGoalsTask {
            output_path: Some(tmp_file.as_ref().to_path_buf()),
        };

        let test_context = TestContext::new();
        let mut execution_context = test_context.execution_context;

        compare_to_monthly_goals_task
            .run(&mut execution_context)
            .unwrap();

        let budget_cfg = execution_context.get_budget_cfg();
        let monthly_goals = &budget_cfg.monthly_goals;

        let needs_amt = test_context.needs_amt;
        let wants_amt = test_context.wants_amt;

        let expected_needs_offset = monthly_goals.get_needs() - needs_amt;
        let expected_wants_offset = monthly_goals.get_wants() - wants_amt;
        let expected_savings = test_context.desposit_amt - needs_amt - wants_amt;
        let expected_savings_offset = expected_savings - monthly_goals.get_savings();

        let expected_weekly_wants_goal_offset = budget_cfg.get_weekly_wants_goal() - wants_amt;

        let expected_output = format!(
            "\nmonthly_needs_spend (goal-actual): ${:.2}\
            \nmonthly_wants_spend (goal-actual): ${:.2}\
            \nmonthly_savings (actual-goal): ${:.2}\
            \nmonthly_savings (absolute): ${:.2}\
            \nweekly_wants_spend (goal-actual): ${:.2}\n",
            expected_needs_offset,
            expected_wants_offset,
            expected_savings_offset,
            expected_savings,
            expected_weekly_wants_goal_offset,
        );

        let found_output = std::fs::read_to_string(output_path).unwrap();
        assert_eq!(expected_output, found_output);
    }

    #[test]
    fn test_run_compare_to_monthly_goals_no_file_write() {
        let compare_to_monthly_goals_task = CompareToMonthlyGoalsTask { output_path: None };
        let mut test_context = TestContext::new();
        compare_to_monthly_goals_task
            .run(&mut test_context.execution_context)
            .unwrap();
    }

    #[test]
    fn test_get_output_string() {
        let needs = 100.00;
        let wants = 50.23;
        let savings = 1000.34;
        let monthly_offsets = MonthlyMetrcs::new(needs, wants, savings);
        let weekly_wants_spend_offset = 5.23;
        let expected = format!(
            "\nmonthly_needs_spend (goal-actual): ${:.2}\
            \nmonthly_wants_spend (goal-actual): ${:.2}\
            \nmonthly_savings (actual-goal): ${:.2}\
            \nmonthly_savings (absolute): ${:.2}\
            \nweekly_wants_spend (goal-actual): ${:.2}\n",
            needs, wants, savings, savings, weekly_wants_spend_offset,
        );
        let found = get_output_string(&monthly_offsets, weekly_wants_spend_offset, savings);

        assert_eq!(found, expected);
    }

    struct TestContext {
        pub execution_context: ExecutionContext,
        pub desposit_amt: f64,
        pub wants_amt: f64,
        pub needs_amt: f64,
    }

    impl TestContext {
        fn new() -> Self {
            let today = NaiveDate::from_ymd_opt(2025, 1, 14).unwrap();
            let id = Uuid::new_v4();
            let description = String::from("description");

            let txn_1_amt = 120.23;
            let txn_1 = Txn {
                id,
                description: description.clone(),
                date: today,
                amount: txn_1_amt,
                category: Category::Deposit,
            };

            let txn_2_amt = 12.23;
            let txn_2 = Txn {
                id,
                description: description.clone(),
                date: today - chrono::Days::new(2),
                amount: txn_2_amt,
                category: Category::Wants,
            };

            let txn_3_amt = 56.23;
            let txn_3 = Txn {
                id,
                description: description.clone(),
                date: today - chrono::Days::new(5),
                amount: txn_3_amt,
                category: Category::Needs,
            };

            let txns = Txns::new(vec![txn_1, txn_2, txn_3], vec![]);
            let monthly_goals = MonthlyGoals::new(150.0, 100.50, 200.25).unwrap();
            let weekly_wants_goal = 100.23;
            let budget_cfg = BudgetCfg::new(monthly_goals, weekly_wants_goal).unwrap();

            Self {
                execution_context: ExecutionContext::new(txns, budget_cfg, Some(today)),
                desposit_amt: txn_1_amt,
                wants_amt: txn_2_amt,
                needs_amt: txn_3_amt,
            }
        }
    }
}
