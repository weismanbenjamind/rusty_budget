use crate::cli::Command;
use crate::date_ops::{
    get_days_left_this_month, get_days_left_this_week_end_of_month_aware, get_first_of_month,
};
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use chrono::{Local, NaiveDate};
use log::info;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UpdateBudgetCfgTask {
    savings_goal: Option<f64>,
    wants_goal: Option<f64>,
    needs_goal: Option<f64>,
    weekly_wants_goal: Option<f64>,

    #[serde(default)] // Default for bool is false
    auto_update_weekly_wants_goal: bool, // * Look into making this field clash with passing weekly wants goal
}

impl Task for UpdateBudgetCfgTask {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError> {
        match cmd {
            Command::UpdateBudgetCfg(update_budget_cfg_cmd) => {
                let task = Self {
                    savings_goal: update_budget_cfg_cmd.savings_goal,
                    wants_goal: update_budget_cfg_cmd.wants_goal,
                    needs_goal: update_budget_cfg_cmd.needs_goal,
                    weekly_wants_goal: update_budget_cfg_cmd.weekly_wants_goal,
                    auto_update_weekly_wants_goal: update_budget_cfg_cmd
                        .auto_update_weekly_wants_goal,
                };
                Ok(task)
            }
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        info!("Updating budget cfg.");

        throw_if_auto_update_and_pass_weekly_wants_goal(
            self.auto_update_weekly_wants_goal,
            self.weekly_wants_goal,
        )?;

        match self.auto_update_weekly_wants_goal {
            true => auto_update_weekly_wants_goal(execution_context, None)?,
            false => {
                if let Some(weekly_wants_goal_update) = self.weekly_wants_goal {
                    execution_context
                        .get_budget_cfg_mutable()
                        .set_weekly_wants_goal(weekly_wants_goal_update)?
                }
            }
        }

        let monthly_goals = &mut execution_context.get_budget_cfg_mutable().monthly_goals;

        if let Some(savings_update) = self.savings_goal {
            monthly_goals.set_savings(savings_update)?
        }

        if let Some(wants_update) = self.wants_goal {
            monthly_goals.set_wants(wants_update)?
        }

        if let Some(needs_update) = self.needs_goal {
            monthly_goals.set_needs(needs_update)?
        }

        info!("Done updating budget cfg.");
        Ok(())
    }
}

fn throw_if_auto_update_and_pass_weekly_wants_goal(
    auto_update_weekly_wants_goal: bool,
    weekly_wants_goal_update: Option<f64>,
) -> Result<(), TaskError> {
    match auto_update_weekly_wants_goal && weekly_wants_goal_update.is_some() {
        true => Err(TaskError::InvalidState(
            "Attempting to auto update weekly wants goal and manually update weekly wants goal."
                .to_string(),
        )),
        false => Ok(()),
    }
}

fn auto_update_weekly_wants_goal(
    execution_context: &mut ExecutionContext,
    today: Option<NaiveDate>,
) -> Result<(), TaskError> {
    let today = today.unwrap_or(Local::now().date_naive());
    let first_of_month = get_first_of_month(today);
    let wants_spend_this_month = execution_context
        .get_txns()
        .get_wants_spend(first_of_month, today);
    let days_left_this_month = get_days_left_this_month(today);
    let days_left_this_week_month_aware = get_days_left_this_week_end_of_month_aware(today);

    let remaining_wants_spend =
        execution_context.get_budget_cfg().monthly_goals.get_wants() - wants_spend_this_month;

    let updated_weekly_wants_goal = match remaining_wants_spend > 0.0 {
        true => {
            (remaining_wants_spend / days_left_this_month as f64)
                * days_left_this_week_month_aware as f64
        }
        false => 0.0,
    };

    execution_context
        .get_budget_cfg_mutable()
        .set_weekly_wants_goal(updated_weekly_wants_goal)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BudgetCfg, MonthlyGoals};
    use crate::cli::UpdateBudgetCfgCmd;
    use crate::txn_ops::{Category, Txn, Txns};
    use uuid::Uuid;

    #[test]
    fn test_from_cmd_valid_update_budget_cfg() {
        let savings_goal = Some(100.23);
        let wants_goal = Some(45.56);
        let needs_goal = Some(56.23);
        let weekly_wants_goal = None;
        let auto_update_weekly_wants_goal = true;

        let cmd = UpdateBudgetCfgCmd {
            savings_goal,
            wants_goal,
            needs_goal,
            weekly_wants_goal,
            auto_update_weekly_wants_goal,
        };

        let cmd = Command::UpdateBudgetCfg(cmd);

        let task = UpdateBudgetCfgTask::from_cmd(cmd).unwrap();

        assert_eq!(task.savings_goal, savings_goal);
        assert_eq!(task.wants_goal, wants_goal);
        assert_eq!(task.needs_goal, needs_goal);
        assert_eq!(task.weekly_wants_goal, weekly_wants_goal);
        assert_eq!(
            task.auto_update_weekly_wants_goal,
            auto_update_weekly_wants_goal
        );
    }

    #[test]
    fn test_from_cmd_invalid_cmd_update_budget_cfg() {
        assert!(
            UpdateBudgetCfgTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_run_no_auto_update_update_budget_cfg() {
        let expected_savings_goal = 100.23;
        let expected_needs_goal = 50.23;
        let expected_weekly_wants_goal = 23.23;

        let task = UpdateBudgetCfgTask {
            savings_goal: Some(expected_savings_goal),
            wants_goal: None,
            needs_goal: Some(expected_needs_goal),
            weekly_wants_goal: Some(expected_weekly_wants_goal),
            auto_update_weekly_wants_goal: false,
        };

        let mut execution_context =
            ExecutionContext::new(Txns::default(), BudgetCfg::default(), None);

        task.run(&mut execution_context).unwrap();

        let budget_cfg = execution_context.get_budget_cfg();
        assert_eq!(
            budget_cfg.monthly_goals.get_savings(),
            expected_savings_goal
        );
        assert_eq!(budget_cfg.monthly_goals.get_needs(), expected_needs_goal);
        assert_eq!(
            budget_cfg.get_weekly_wants_goal(),
            expected_weekly_wants_goal
        );
        assert_eq!(budget_cfg.monthly_goals.get_wants(), 0.00); // Since use BudgetCfg::default() above this should be 0.00
    }

    #[test]
    fn test_run_auto_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 11);
        let days_left_this_month = 21; // 31 days in March
        let days_left_this_week = 4; // 2 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        // Moving monthly goals - monthly goals is copy so it get's copied from the &BudgetCfg borrow in get_budget_cfg()
        let original_monthly_goals = test_context
            .execution_context
            .get_budget_cfg()
            .monthly_goals;

        // Moving exectuion context
        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        );
        assert_eq!(
            original_monthly_goals,
            execution_context.get_budget_cfg().monthly_goals
        );
    }

    #[test]
    fn test_throw_if_auto_update_and_pass_weekly_wants_goal_update_budget_cfg() {
        assert!(throw_if_auto_update_and_pass_weekly_wants_goal(true, Some(32.2)).is_err());
        assert!(throw_if_auto_update_and_pass_weekly_wants_goal(false, Some(32.2)).is_ok());
        assert!(throw_if_auto_update_and_pass_weekly_wants_goal(true, None).is_ok());
    }

    #[test]
    fn test_update_weekly_wants_goal_already_over_goal_update_budget_cfg() {
        let budget_cfg = BudgetCfg::new(MonthlyGoals::default(), 100.00).unwrap();
        let today = get_naive_date(2026, 1, 22);

        let amount = 125.00;
        let txn = Txn {
            id: Uuid::new_v4(),
            date: today,
            description: "test".to_string(),
            amount: amount,
            category: Category::Wants,
        };

        let txns = Txns::new(vec![txn], vec!["header".to_string()]);
        let mut execution_context = ExecutionContext::new(txns, budget_cfg, None);
        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            0.00,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        );
    }

    #[test]
    fn test_defaults_update_budget_cfg() {
        let yml = r#"
        savings-goal: 1234.45
        weekly-wants-goal: 100.67
        "#;

        let update_cfg_task: UpdateBudgetCfgTask = serde_yaml::from_str(yml).unwrap();
        assert_eq!(update_cfg_task.savings_goal, Some(1234.45));
        assert!(update_cfg_task.wants_goal.is_none());
        assert!(update_cfg_task.needs_goal.is_none());
        assert_eq!(update_cfg_task.weekly_wants_goal, Some(100.67));
    }

    #[test]
    fn test_update_weekly_wants_goal_less_than_a_week_in_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 30);
        let days_left_this_month = 2; // 31 days in March
        let days_left_this_week = 2; // 2 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        );
    }

    #[test]
    fn test_update_weekly_wants_goal_1_week_left_in_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 24);
        let days_left_this_month = 8; // 31 days in March. Since we use the 24th and our calcs are inclusive expect 8 days left in month
        let days_left_this_week = 5; // 5 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        );
    }

    #[test]
    fn test_update_weekly_wants_goal_1_week_and_1_day_left_in_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 23);
        let days_left_this_month = 9; // 31 days in March. Since we use the 23rd and our calcs are inclusive expect 9 days left in month
        let days_left_this_week = 6; // 6 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        );
    }

    #[test]
    fn test_update_weekly_wants_goal_1_day_less_than_week_left_in_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 25);
        let days_left_this_month = 7; // 31 days in March. Since we use the 25th and our calcs are inclusive expect 7 days left in month
        let days_left_this_week = 4; // 4 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        )
    }

    #[test]
    fn test_update_weekly_wants_goal_2_day_less_than_week_left_in_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 26);
        let days_left_this_month = 6; // 31 days in March. Since we use the 26th and our calcs are inclusive expect 6 days left in month
        let days_left_this_week = 3; // 3 days left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        )
    }

    #[test]
    fn test_update_weekly_wants_goal_last_day_of_month_update_budget_cfg() {
        let today = get_naive_date(2026, 3, 31);
        let days_left_this_month = 1; // 31 days in March. Since we use the 31st and our calcs are inclusive expect 1 day left in month
        let days_left_this_week = 1; // 1 day left this week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        )
    }

    #[test]
    fn test_update_weekly_wants_goal_days_left_in_month_and_week_equal_update_budget_cfg() {
        let today = get_naive_date(2026, 2, 25);
        let days_left_this_month = 4; // 28 days in Feb 2026. Since we use the 25th and our calcs are inclusive expect 4 days left in the month
        let days_left_this_week = 4; // In this case should also have 1 day left in the week
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        )
    }

    #[test]
    fn test_update_weekly_wants_goal_first_week_of_month_starts_mid_week_update_budget_cfg() {
        let today = get_naive_date(2026, 4, 2);
        let days_left_this_month = 29; // 30 days in April 2026. Since we use the 2nd and our calcs are inclusive expect 29 days left in the month
        let days_left_this_week = 3; // 3 days left in week. Including Thursday the 2nd
        let test_context = TestContext::new(today);

        let expected = test_context
            .get_expected_weekly_wants_goal_auto_update(days_left_this_month, days_left_this_week);

        let mut execution_context = test_context.execution_context;

        auto_update_weekly_wants_goal(&mut execution_context, Some(today)).unwrap();

        assert_eq!(
            expected,
            execution_context.get_budget_cfg().get_weekly_wants_goal()
        )
    }

    fn get_naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    struct TestContext {
        pub weekly_wants_spend_goal_minus_actual: f64,
        pub execution_context: ExecutionContext,
    }

    impl TestContext {
        pub fn new(txn_date: NaiveDate) -> Self {
            let monthly_goals = MonthlyGoals::new(100.00, 100.00, 100.00).unwrap();
            let original_wants_goal = 100.00;
            let budget_cfg = BudgetCfg::new(monthly_goals, original_wants_goal).unwrap();

            let wants_spend_up_to_txn_date = 50.00;
            let txn = Txn {
                id: Uuid::new_v4(),
                description: "test".to_string(),
                date: txn_date,
                amount: wants_spend_up_to_txn_date,
                category: Category::Wants,
            };

            let txns = Txns::new(
                Vec::<Txn>::from([txn]),
                Vec::<String>::from([String::default()]),
            );

            let execution_context = ExecutionContext::new(txns, budget_cfg, None);

            let weekly_wants_spend_goal_minus_actual =
                original_wants_goal - wants_spend_up_to_txn_date;

            Self {
                weekly_wants_spend_goal_minus_actual,
                execution_context,
            }
        }

        pub fn get_expected_weekly_wants_goal_auto_update(
            &self,
            days_left_this_month: i32,
            days_left_this_week: i32,
        ) -> f64 {
            return self.weekly_wants_spend_goal_minus_actual / days_left_this_month as f64
                * days_left_this_week as f64;
        }
    }
}
