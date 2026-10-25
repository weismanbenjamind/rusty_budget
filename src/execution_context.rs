use crate::cfg::BudgetCfg;
use crate::date_ops::today as get_today;
use crate::txn_ops::Txns;
use chrono::NaiveDate;

#[derive(Debug)]
pub enum ExecutionContextError {
    Base(String),
    FromFileError(String),
    ConversionError(String),
}

impl std::error::Error for ExecutionContextError {}

impl std::fmt::Display for ExecutionContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Base(err_msg) | Self::FromFileError(err_msg) | Self::ConversionError(err_msg) => {
                write!(f, "{err_msg}")
            }
        }
    }
}

impl From<anyhow::Error> for ExecutionContextError {
    fn from(err: anyhow::Error) -> ExecutionContextError {
        ExecutionContextError::Base(format!(
            "Failure generating execution context. Error message: {err}"
        ))
    }
}

pub struct ExecutionContext {
    txns: Txns,
    budget_cfg: BudgetCfg,
    today: NaiveDate, // ? What is this used for? - Used to inject a date into a .run() call for testing. Not used in practice -> Test seam
}

impl ExecutionContext {
    pub fn new(txns: Txns, budget_cfg: BudgetCfg, today: Option<NaiveDate>) -> Self {
        Self {
            txns,
            budget_cfg,
            today: get_default_today(today),
        }
    }

    pub fn get_txns_mutable(&mut self) -> &mut Txns {
        &mut self.txns
    }

    pub fn get_txns(&self) -> &Txns {
        &self.txns
    }

    pub fn get_budget_cfg(&self) -> &BudgetCfg {
        &self.budget_cfg
    }

    pub fn get_today(&self) -> NaiveDate {
        self.today
    }

    pub fn get_budget_cfg_mutable(&mut self) -> &mut BudgetCfg {
        &mut self.budget_cfg
    }
}

fn get_default_today(today: Option<NaiveDate>) -> NaiveDate {
    today.unwrap_or(get_today())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::MonthlyGoals;
    use crate::txn_ops::{Category, Txn};
    use chrono::{Local, NaiveDate};
    use uuid::Uuid;

    const TEST_DATE: NaiveDate = NaiveDate::from_ymd_opt(2024, 2, 18).unwrap();

    #[test]
    fn test_construct() {
        let txn = Txn {
            date: NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
            description: "test".to_string(),
            amount: 100.23,
            id: Uuid::new_v4(),
            category: Category::Wants,
        };
        let txns = Txns::new(vec![txn], vec!["header".to_string()]);
        let monthly_goals = MonthlyGoals::new(23.0, 23.0, 23.0).unwrap();
        let budget_cfg = BudgetCfg::new(monthly_goals, 23.2).unwrap();

        let execution_context = ExecutionContext {
            txns: txns.clone(),
            budget_cfg: budget_cfg.clone(),
            today: TEST_DATE,
        };

        assert_eq!(execution_context.txns, txns);
        assert_eq!(execution_context.budget_cfg, budget_cfg);
        assert_eq!(execution_context.today, TEST_DATE);
    }

    #[test]
    fn test_new() {
        let txn = Txn {
            date: NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
            description: "test".to_string(),
            amount: 100.23,
            id: Uuid::new_v4(),
            category: Category::Wants,
        };
        let txns = Txns::new(vec![txn.clone()], vec!["header".to_string()]);
        let monthly_goals = MonthlyGoals::new(23.0, 23.0, 23.0).unwrap();
        let budget_cfg = BudgetCfg::new(monthly_goals, 23.2).unwrap();

        let execution_context = ExecutionContext::new(txns, budget_cfg.clone(), Some(TEST_DATE));

        let found_txns = execution_context.txns.get_all_txns();
        assert_eq!(found_txns.len(), 1);
        let found_txn = &found_txns[0];
        assert_eq!(*found_txn, txn);

        assert_eq!(execution_context.budget_cfg, budget_cfg);
    }

    #[test]
    fn test_get_default_today_some() {
        let date = NaiveDate::from_ymd_opt(2023, 3, 15).unwrap();
        let found = get_default_today(Some(date));
        assert_eq!(found, date);

        assert_eq!(get_default_today(None), Local::now().date_naive());
    }

    #[test]
    fn test_get_default_today_none() {
        assert_eq!(get_default_today(None), Local::now().date_naive());
    }

    #[test]
    fn test_getters_execution_context() {
        let txn = Txn {
            date: NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
            description: "test".to_string(),
            amount: 100.23,
            id: Uuid::new_v4(),
            category: Category::Wants,
        };
        let txns = Txns::new(vec![txn.clone()], vec!["header".to_string()]);

        let mut execution_context = ExecutionContext {
            txns,
            budget_cfg: BudgetCfg::default(),
            today: TEST_DATE,
        };

        let expected_budget_cfg = BudgetCfg::default();

        assert_eq!(execution_context.get_txns().len(), 1);
        assert_eq!(execution_context.get_txns().get_all_txns()[0], txn);
        assert_eq!(execution_context.get_txns_mutable().get_all_txns()[0], txn);
        assert_eq!(*execution_context.get_budget_cfg(), expected_budget_cfg);
        assert_eq!(
            *execution_context.get_budget_cfg_mutable(),
            expected_budget_cfg
        );
        assert_eq!(execution_context.get_today(), TEST_DATE);
    }

    #[test]
    fn test_exection_context_error_display_exection_context() {
        let err_msg = "msg";
        let base = ExecutionContextError::Base(err_msg.to_string());
        let from_file_error = ExecutionContextError::FromFileError(err_msg.to_string());
        let conversion_error = ExecutionContextError::ConversionError(err_msg.to_string());

        assert_eq!(format!("{base}"), err_msg);
        assert_eq!(format!("{from_file_error}"), err_msg);
        assert_eq!(format!("{conversion_error}"), err_msg);
    }

    #[test]
    fn test_from_anyhow_execution_context() {
        let msg = "grrr";
        assert_eq!(
            ExecutionContextError::from(anyhow::anyhow!(msg)).to_string(),
            format!(
                "Failure generating execution context. Error message: {}",
                msg
            )
        )
    }
}
