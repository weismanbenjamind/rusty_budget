use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use crate::txn_ops::{Category, Txn};
use chrono::NaiveDate;
use log::info;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UpdateTxnsTask {
    id: uuid::Uuid,
    date: Option<chrono::NaiveDate>,
    description: Option<String>,
    amount: Option<f64>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    category: Option<Category>,

    #[serde(default)] // Default for bool is false
    sort: bool,
}

impl Task for UpdateTxnsTask {
    fn from_cmd(cmd: crate::cli::Command) -> Result<Self, TaskError> {
        match cmd {
            Command::UpdateTxn(update_txn_cmd) => Ok(Self {
                id: update_txn_cmd.id,
                description: update_txn_cmd.description,
                amount: update_txn_cmd.amount,
                date: update_txn_cmd.date,
                category: update_txn_cmd.category,
                sort: update_txn_cmd.sort,
            }),
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        let txns = execution_context.get_txns_mutable();
        let txn = txns.get_by_id_mut(self.id)?;
        let id = txn.id;

        info!("Updating txn with id {id} based on task spec {:.?}", self);

        update_txn_inplace(txn, self.description, self.amount, self.date, self.category);

        if self.sort {
            txns.sort();
        }

        info!("Done updating txn with id: {id}");

        Ok(())
    }
}

fn update_txn_inplace(
    txn: &mut Txn,
    description: Option<String>,
    amount: Option<f64>,
    date: Option<NaiveDate>,
    category: Option<Category>,
) {
    if let Some(description_update) = description {
        txn.description = description_update
    }

    if let Some(amount_update) = amount {
        txn.amount = amount_update
    }

    if let Some(date_update) = date {
        txn.date = date_update
    }

    if let Some(category_upate) = category {
        txn.category = category_upate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::BudgetCfg;
    use crate::cli::UpdateTxnCmd;
    use crate::txn_ops::{Txn, Txns};
    use uuid::Uuid;

    #[test]
    fn test_defaults_update_txns() {
        let id = uuid::Uuid::new_v4();
        let yml = format!("id: {}\namount: 100.45", id);
        let update_txns_task: UpdateTxnsTask = serde_yaml::from_str(&yml).unwrap();

        assert_eq!(update_txns_task.id, id);
        assert!(update_txns_task.description.is_none());
        assert_eq!(update_txns_task.amount, Some(100.45));
        assert!(update_txns_task.date.is_none());
        assert!(update_txns_task.category.is_none());
        assert_eq!(update_txns_task.sort, false);
    }

    #[test]
    fn test_invalid_category_update_txns() {
        let id = uuid::Uuid::new_v4();
        assert!(
            serde_yaml::from_str::<UpdateTxnsTask>(&format!("id: {}\ncategory: bad", id)).is_err()
        )
    }

    #[test]
    fn test_category_case_sensitivity_update_txns() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(
            serde_yaml::from_str::<UpdateTxnsTask>(&format!("id: {}\ncategory: WaNtS", id))
                .unwrap()
                .category,
            Some(Category::Wants)
        )
    }

    #[test]
    fn test_from_cmd_update_txs() {
        let cmd = UpdateTxnCmd {
            id: Uuid::new_v4(),
            date: None,
            description: None,
            amount: Some(11.34),
            category: None,
            sort: true,
        };

        let cmd_dispatch = Command::UpdateTxn(cmd.clone());
        let task = UpdateTxnsTask::from_cmd(cmd_dispatch).unwrap();

        assert_eq!(task.id, cmd.id);
        assert_eq!(task.date, cmd.date);
        assert_eq!(task.description, cmd.description);
        assert_eq!(task.amount, cmd.amount);
        assert_eq!(task.category, cmd.category);
        assert_eq!(task.sort, cmd.sort);
    }

    #[test]
    fn test_from_cmd_invalid_update_txns() {
        assert!(
            UpdateTxnsTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_update_txn_inplace_update_txns() {
        let id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 3, 3).unwrap();
        let description = "desc";
        let amount = 110.32;
        let category = Category::Deposit;

        let mut txn = Txn {
            id,
            date,
            description: description.to_string() + "_suffix",
            amount: amount + 50.23,
            category: category,
        };

        update_txn_inplace(
            &mut txn,
            Some(description.to_string()),
            Some(amount),
            None,
            None,
        );

        assert_eq!(txn.id, id);
        assert_eq!(txn.date, date);
        assert_eq!(txn.description.as_str(), description);
        assert_eq!(txn.amount, amount);
        assert_eq!(txn.category, category);
    }

    #[test]
    fn test_run_sort_update_txns() {
        let test_context = TestContext::new(true);

        // Move these out of the test_context struct - test_context is now partaially moved
        let task = test_context.task;
        let mut execution_context = test_context.execution_context;

        task.run(&mut execution_context).unwrap();

        let found_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(found_txns.len(), 2);

        // ! Note - order of txns matters in the txns field of the execution context struct since we are testing sorting
        let original_older_txn = test_context.original_older_txn;
        let updated_older_txn = &found_txns[1];

        assert_eq!(updated_older_txn.id, original_older_txn.id);
        assert_eq!(updated_older_txn.amount, test_context.updated_amt);
        assert_eq!(
            updated_older_txn.description,
            original_older_txn.description
        );
        assert_eq!(updated_older_txn.date, original_older_txn.date);
        assert_eq!(updated_older_txn.category, original_older_txn.category);

        assert_eq!(&found_txns[0], &test_context.newer_txn);
    }

    #[test]
    fn test_run_no_sort_update_txns() {
        let test_context = TestContext::new(false);

        // Move out of test context. Test context is no partially moved
        let task = test_context.task;
        let mut execution_context = test_context.execution_context;

        task.run(&mut execution_context).unwrap();

        let found_txns = execution_context.get_txns().get_all_txns();
        assert_eq!(found_txns.len(), 2);

        // ! Note - order of txns matters in the txns field of the execution context struct since we are testing sorting
        let original_older_txn = test_context.original_older_txn;
        let updated_older_txn = &found_txns[0];

        assert_eq!(original_older_txn.id, updated_older_txn.id);
        assert_eq!(original_older_txn.date, updated_older_txn.date);
        assert_eq!(
            original_older_txn.description,
            updated_older_txn.description
        );
        assert_eq!(test_context.updated_amt, updated_older_txn.amount);
        assert_eq!(original_older_txn.category, updated_older_txn.category);

        assert_eq!(&found_txns[1], &test_context.newer_txn);
    }

    struct TestContext {
        execution_context: ExecutionContext,
        task: UpdateTxnsTask,
        original_older_txn: Txn,
        newer_txn: Txn,
        updated_amt: f64,
    }

    impl TestContext {
        fn new(sort: bool) -> Self {
            let older_txn_id = Uuid::new_v4();
            let older_txn_amount = 134.34;

            let older_txn = Txn {
                id: older_txn_id,
                date: NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(),
                description: String::from("older_txn"),
                amount: older_txn_amount,
                category: Category::Deposit,
            };

            // Note - order matters here
            let newer_txn = Txn {
                id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
                description: String::from("newer_txn"),
                amount: 4.30,
                category: Category::Wants,
            };

            // ! Note - order matters here since we are sorting. The new txn should be the first txn after sort
            let txns: Vec<Txn> = Vec::from([older_txn.clone(), newer_txn.clone()]);
            let header: Vec<String> = Vec::from([String::default()]);
            let txns = Txns::new(txns, header);

            let new_amt = older_txn_amount + 23.23;

            let cmd = UpdateTxnCmd {
                id: older_txn_id,
                date: None,
                description: None,
                amount: Some(new_amt),
                category: None,
                sort: sort,
            };

            let execution_context = ExecutionContext::new(txns, BudgetCfg::default(), None);

            let task = UpdateTxnsTask {
                id: cmd.id,
                date: cmd.date,
                description: cmd.description,
                amount: cmd.amount,
                category: cmd.category,
                sort: cmd.sort,
            };

            Self {
                execution_context: execution_context,
                task: task,
                original_older_txn: older_txn,
                newer_txn: newer_txn,
                updated_amt: new_amt,
            }
        }
    }
}
