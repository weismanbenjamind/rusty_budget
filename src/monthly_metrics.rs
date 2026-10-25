use crate::cfg::MonthlyGoals;
use crate::date_ops::get_first_of_month;
use crate::txn_ops::{Category, Txns};
use chrono::NaiveDate;

#[derive(PartialEq, Debug)]
pub struct MonthlyMetrcs {
    needs: f64,
    wants: f64,
    savings: f64,
}

impl MonthlyMetrcs {
    #[allow(dead_code)]
    pub fn new(needs: f64, wants: f64, savings: f64) -> MonthlyMetrcs {
        MonthlyMetrcs {
            needs,
            wants,
            savings,
        }
    }

    pub fn needs(&self) -> f64 {
        self.needs
    }

    pub fn wants(&self) -> f64 {
        self.wants
    }

    pub fn savings(&self) -> f64 {
        self.savings
    }

    pub fn from_txns(txns: &Txns, until: NaiveDate) -> Self {
        let mut deposits = 0.0;
        let mut wants = 0.0;
        let mut needs = 0.0;

        let since = get_first_of_month(until);

        for txn in txns.get(since, until, false) {
            match txn.category {
                Category::Deposit => deposits += txn.amount,
                Category::Needs => needs += txn.amount,
                Category::Wants => wants += txn.amount,
            }
        }

        let savings = deposits - wants - needs;

        Self {
            needs,
            wants,
            savings,
        }
    }

    pub fn into_offsets(self, monthly_goals: &MonthlyGoals) -> Self {
        Self {
            savings: self.savings - monthly_goals.get_savings(),
            wants: monthly_goals.get_wants() - self.wants,
            needs: monthly_goals.get_needs() - self.needs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txn_ops::{Category, Txn};
    use chrono::Datelike;
    use uuid::Uuid;

    #[test]
    fn test_new() {
        let savings = 101.2;
        let wants = 12.34;
        let needs = 78.98;

        let mm = MonthlyMetrcs::new(needs, wants, savings);
        let expected = MonthlyMetrcs {
            needs,
            wants,
            savings,
        };

        assert_eq!(mm, expected);
    }

    #[test]
    fn test_monthly_metrics_from_txns() {
        let id = Uuid::new_v4();
        let until = chrono::NaiveDate::from_ymd_opt(2025, 1, 18).unwrap();
        let description = String::from("description");

        let txn_1_amt = 120.23;
        let txn_1 = Txn {
            id,
            description: description.clone(),
            date: until,
            amount: txn_1_amt,
            category: Category::Deposit,
        };

        let txn_2_amt = 12.23;
        let txn_2 = Txn {
            id,
            description: description.clone(),
            date: until - chrono::Days::new(3),
            amount: txn_2_amt,
            category: Category::Wants,
        };

        let txn_3_amt = 56.23;
        let txn_3 = Txn {
            id,
            description: description.clone(),
            date: until - chrono::Days::new(5),
            amount: txn_3_amt,
            category: Category::Needs,
        };

        let txn_4 = Txn {
            id,
            description: description.clone(),
            date: until.with_month(2).unwrap(),
            amount: 100.45,
            category: Category::Deposit,
        };

        let txns = Txns::new(
            vec![txn_1, txn_2, txn_3, txn_4],
            vec![String::from("header")],
        );

        let found = MonthlyMetrcs::from_txns(&txns, until);
        assert_eq!(found.needs, txn_3_amt);
        assert_eq!(found.wants, txn_2_amt);
        assert_eq!(found.savings, txn_1_amt - txn_2_amt - txn_3_amt);
    }

    #[test]
    fn test_get_offsets() {
        let wants = 45.56;
        let needs = 100.23;
        let savings = 78.45;

        let monthly_metrics = MonthlyMetrcs {
            needs,
            wants,
            savings,
        };

        let monthly_goals = MonthlyGoals::new(150.0, 100.50, 200.25).unwrap();

        let offsets = monthly_metrics.into_offsets(&monthly_goals);

        assert_eq!(offsets.needs, monthly_goals.get_needs() - needs);
        assert_eq!(offsets.savings, savings - monthly_goals.get_savings());
        assert_eq!(offsets.wants, monthly_goals.get_wants() - wants);
    }
}
