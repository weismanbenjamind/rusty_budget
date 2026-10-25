use crate::cfg::MonthlyGoals;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BudgetCfg {
    pub monthly_goals: MonthlyGoals,
    weekly_wants_goal: f64,
}

impl BudgetCfg {
    #[allow(dead_code)]
    pub fn new(monthly_goals: MonthlyGoals, weekly_wants_goal: f64) -> Result<Self> {
        validate_goal(weekly_wants_goal)?;
        Ok(Self {
            monthly_goals,
            weekly_wants_goal,
        })
    }

    pub fn set_weekly_wants_goal(&mut self, goal: f64) -> Result<()> {
        validate_goal(goal)?;
        self.weekly_wants_goal = goal;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_weekly_wants_goal(&self) -> f64 {
        self.weekly_wants_goal
    }

    // TODO - Fix newline on output - weekly wants spend should have a when outputing
    pub fn to_yml(&self, path: &str) -> Result<()> {
        let output = format!(
            "monthly_goals:\n  savings: {:.2}\n  needs: {:.2}\n  wants: {:2}\n\nweekly_wants_goal: {:2}\n",
            self.monthly_goals.get_savings(),
            self.monthly_goals.get_needs(),
            self.monthly_goals.get_wants(),
            self.weekly_wants_goal
        );

        write(path, output).expect("Failed to write budget cfg to {path}.");

        Ok(())
    }

    // TODO - Need a more descriptive error message if path does not exist
    pub fn from_yml(path: &str) -> Result<Self> {
        let budget_cfg: BudgetCfg = serde_yaml::from_str(&read_to_string(path)?).map_err(|e| {
            anyhow::anyhow!("Failed to parse budget cfg at {:?}. Error {:?}", path, e)
        })?;

        budget_cfg.monthly_goals.validate()?;
        validate_goal(budget_cfg.weekly_wants_goal)?;

        Ok(budget_cfg)
    }
}

pub fn validate_goal(goal: f64) -> Result<()> {
    match goal >= 0f64 {
        true => Ok(()),
        false => Err(anyhow::anyhow!(
            "Goal must be geater than or equal to zero. Found {}",
            goal
        )),
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_from_yml_valid() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(
            temp_file,
            "monthly_goals:\n  needs: 4514.84\n  wants: 809.28\n  savings: 1000\n\nweekly_wants_goal: 150.23\n"
        )?;

        let cfg = BudgetCfg::from_yml(temp_file.path().to_str().unwrap())?;
        assert_eq!(cfg.monthly_goals.get_savings(), 1000.00);
        assert_eq!(cfg.monthly_goals.get_needs(), 4514.84);
        assert_eq!(cfg.monthly_goals.get_wants(), 809.28);
        assert_eq!(cfg.weekly_wants_goal, 150.23);

        Ok(())
    }

    #[test]
    fn test_from_yml_file_does_not_exist() {
        let result = BudgetCfg::from_yml("this/does/not/exist");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No such file or directory")
        );
    }

    #[test]
    fn test_from_yml_invalid_yml() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "Some invalid yml")?;
        let result = BudgetCfg::from_yml(&temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse budget cfg at")
        );
        Ok(())
    }

    #[test]
    fn test_from_yml_negative_val() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "monthly_goals:\n  needs: -1.23\n  wants: 34.23\n  savings: 34.35\n\nweekly_wants_goal: 150.23\n"
        )
        .unwrap();
        let cfg = BudgetCfg::from_yml(temp_file.path().to_str().unwrap());
        assert!(cfg.is_err());
        assert!(
            cfg.unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("goal must be geater than or equal to zero")
        );
    }

    #[test]
    fn test_to_yml() {
        let savings_goal = 23.45;
        let wants_goal = 23.54;
        let needs_goal = 34.67;
        let monthly_goals = MonthlyGoals::new(savings_goal, needs_goal, wants_goal).unwrap();
        let weekly_wants_goal = 150.23;

        let budget_cfg = BudgetCfg {
            monthly_goals,
            weekly_wants_goal,
        };
        let tempfile = NamedTempFile::with_suffix(".yml").unwrap();
        let cfg_path = tempfile.path().to_str().unwrap();

        budget_cfg.to_yml(cfg_path).unwrap();
        let contents = std::fs::read_to_string(cfg_path).unwrap();

        let expected_contents = format!(
            "monthly_goals:\n  savings: {}\n  needs: {}\n  wants: {}\n\nweekly_wants_goal: {}\n",
            savings_goal, needs_goal, wants_goal, weekly_wants_goal
        );

        assert!(contents.contains(&expected_contents));
    }

    #[test]
    fn test_new() {
        let monthly_goals = MonthlyGoals::new(123.45, 230.34, 234.56).unwrap();
        let weekly_wants_goal = 1234.45;
        let budget_cfg = BudgetCfg::new(monthly_goals, weekly_wants_goal).unwrap();
        assert_eq!(budget_cfg.monthly_goals, monthly_goals);
        assert_eq!(budget_cfg.weekly_wants_goal, weekly_wants_goal);
    }

    #[test]
    fn test_new_err() {
        let monthly_goals = MonthlyGoals::new(123.45, 230.34, 234.56).unwrap();
        let weekly_wants_goal = -1234.45;
        let result = BudgetCfg::new(monthly_goals, weekly_wants_goal);
        assert!(result.is_err());
    }

    #[test]
    fn test_weekly_wants_goal_getter_and_setter() {
        let monthly_goals = MonthlyGoals::new(123.45, 230.34, 234.56).unwrap();
        let weekly_wants_goal = 1234.45;
        let mut budget_cfg = BudgetCfg::new(monthly_goals, weekly_wants_goal).unwrap();
        let new_weekly_wants_goal = weekly_wants_goal + 239.34;
        budget_cfg
            .set_weekly_wants_goal(new_weekly_wants_goal)
            .unwrap();
        assert_eq!(budget_cfg.get_weekly_wants_goal(), new_weekly_wants_goal);
    }

    #[test]
    fn test_set_weekly_wants_goal_err() {
        let monthly_goals = MonthlyGoals::new(123.45, 230.34, 234.56).unwrap();
        let weekly_wants_goal = 1234.45;
        let mut budget_cfg = BudgetCfg::new(monthly_goals, weekly_wants_goal).unwrap();
        let new_weekly_wants_goal = -123.34;
        let result = budget_cfg.set_weekly_wants_goal(new_weekly_wants_goal);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_goal() {
        assert!(validate_goal(-1.23).is_err());
        assert!(validate_goal(0.00).is_ok());
        assert!(validate_goal(123.34).is_ok());
    }
}
