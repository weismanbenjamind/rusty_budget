use crate::cfg::validate_goal;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonthlyGoalsError {
    #[error("{0}")]
    Base(String),
}

impl From<anyhow::Error> for MonthlyGoalsError {
    fn from(value: anyhow::Error) -> Self {
        Self::Base(value.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MonthlyGoal {
    Savings,
    Needs,
    Wants,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Default)]
pub struct MonthlyGoals {
    savings: f64,
    needs: f64,
    wants: f64,
}

impl MonthlyGoals {
    #[allow(dead_code)]
    pub fn new(savings: f64, needs: f64, wants: f64) -> Result<Self> {
        validate_goal(savings)?;
        validate_goal(needs)?;
        validate_goal(wants)?;

        Ok(Self {
            savings,
            needs,
            wants,
        })
    }

    pub fn get_savings(&self) -> f64 {
        self.savings
    }

    pub fn get_needs(&self) -> f64 {
        self.needs
    }

    pub fn get_wants(&self) -> f64 {
        self.wants
    }

    pub fn set_savings(&mut self, val: f64) -> Result<(), MonthlyGoalsError> {
        self.set_goal(val, MonthlyGoal::Savings)?;
        Ok(())
    }

    pub fn set_wants(&mut self, val: f64) -> Result<(), MonthlyGoalsError> {
        self.set_goal(val, MonthlyGoal::Wants)?;
        Ok(())
    }

    pub fn set_needs(&mut self, val: f64) -> Result<(), MonthlyGoalsError> {
        self.set_goal(val, MonthlyGoal::Needs)?;
        Ok(())
    }

    pub fn set_goal(&mut self, val: f64, goal: MonthlyGoal) -> Result<()> {
        validate_goal(val)?;
        match goal {
            MonthlyGoal::Savings => self.savings = val,
            MonthlyGoal::Needs => self.needs = val,
            MonthlyGoal::Wants => self.wants = val,
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        validate_goal(self.savings)?;
        validate_goal(self.needs)?;
        validate_goal(self.wants)?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_set_goal() {
        let mut goals = MonthlyGoals {
            savings: 100f64,
            needs: 200f64,
            wants: 234f64,
        };

        let target_savings = goals.savings + 34.65;
        let target_needs = goals.needs + 3879.34;
        let target_wants = goals.wants + 4738.67;

        goals
            .set_goal(target_savings, MonthlyGoal::Savings)
            .unwrap();

        goals.set_goal(target_needs, MonthlyGoal::Needs).unwrap();
        goals.set_goal(target_wants, MonthlyGoal::Wants).unwrap();

        assert!(goals.savings == target_savings);
        assert!(goals.needs == target_needs);
        assert!(goals.wants == target_wants);
    }

    #[test]
    fn test_validate() {
        let mut monthly_goals = MonthlyGoals {
            savings: 100.23,
            needs: -23.44,
            wants: 23.56,
        };
        assert!(monthly_goals.validate().is_err());

        monthly_goals = MonthlyGoals {
            savings: -100.23,
            needs: 23.44,
            wants: 23.56,
        };
        assert!(monthly_goals.validate().is_err());

        monthly_goals = MonthlyGoals {
            savings: 100.23,
            needs: 23.44,
            wants: -23.56,
        };
        assert!(monthly_goals.validate().is_err());
    }
}
