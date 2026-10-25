mod budget_cfg;
mod monthy_goals;

pub use budget_cfg::BudgetCfg;
use budget_cfg::validate_goal;

// Allow the unused import on MonthlyGoals. Public function off the struct that would benefit from having MonthlyGoal public if needed
#[allow(unused_imports)]
pub use monthy_goals::{MonthlyGoal, MonthlyGoals, MonthlyGoalsError};
