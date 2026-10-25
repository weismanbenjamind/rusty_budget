use crate::cli::Command;
use crate::execution_context::ExecutionContext;
use crate::job_ops::tasks::task::{Task, TaskError};
use crate::txn_ops::Category;
use log::info;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct OffsetCalcTask {
    checking_balance: f64,
    freedom_unlimited_balance: f64,
    discover_balance: f64,
    huntington_balance: f64,
    sofi_balance: f64,
    wallet_balance: f64,

    #[serde(default)] // Default for bool is false
    sort: bool,
}

impl Task for OffsetCalcTask {
    fn from_cmd(cmd: Command) -> Result<Self, TaskError> {
        match cmd {
            Command::OffsetCalc(offset_calc_cmd) => Ok(Self {
                checking_balance: offset_calc_cmd.checking_balance,
                freedom_unlimited_balance: offset_calc_cmd.freedom_unlimited_balance,
                discover_balance: offset_calc_cmd.discover_balance,
                huntington_balance: offset_calc_cmd.huntington_balance,
                sofi_balance: offset_calc_cmd.sofi_balance,
                wallet_balance: offset_calc_cmd.wallet_balance,
                sort: offset_calc_cmd.sort,
            }),
            _ => Err(TaskError::FromCmdError(cmd.into())),
        }
    }

    fn run(self, execution_context: &mut ExecutionContext) -> Result<(), TaskError> {
        info!("Doing offset calculation");

        let current_worth = self.checking_balance
            + self.huntington_balance
            + self.sofi_balance
            + self.wallet_balance
            - self.freedom_unlimited_balance
            - self.discover_balance;

        let calculated_worth =
            execution_context
                .get_txns()
                .get_all_txns()
                .iter()
                .fold(0.00, |acc, txn| match txn.category {
                    Category::Deposit => acc + txn.amount,
                    Category::Needs | Category::Wants => acc - txn.amount,
                });

        // Need offset < 0.
        // This means our calculated is greater than our actual
        // In this case we need to add needs to reduce are calculated to our actual
        let offset_amount = current_worth - calculated_worth;

        if offset_amount == 0.00 {
            info!("Txns aligned offset of $0.00 found");
            return Ok(());
        }

        // Note - we know offset amount is not equal to zero due to return statement above
        let offset_category = match offset_amount > 0.00 {
            true => Category::Deposit,
            false => Category::Needs,
        };
        let date = chrono::Local::now().date_naive();
        let description = "offset";
        let offset_amount = offset_amount.abs(); // * Shadowing offset amount here

        info!(
            "Offseting by amount ${} to category {}",
            offset_amount, offset_category
        );

        let txns = execution_context.get_txns_mutable();

        txns.add(
            date,
            description.to_string(),
            offset_amount.abs(),
            offset_category,
        );

        if self.sort {
            txns.sort();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tets {
    use super::*;

    use crate::cfg::BudgetCfg;
    use crate::cli::{Command, OffsetCalcCmd};
    use crate::txn_ops::{Txn, Txns};
    use uuid::Uuid;

    const HEADER: [&str; 5] = ["id", "date", "description", "amount", "category"];
    const CHECKING_BALANCE: f64 = 10.00;
    const FREEDOM_UNLIMITED_BALANCE: f64 = 1.00;
    const DISCOVER_BALANCE: f64 = 10.00;
    const HUNTINGTON_BALANCE: f64 = 10.00;
    const SOFI_BALANCE: f64 = 10.00;
    const WALLET_BALANCE: f64 = 10.00;
    const MOCK_CURRENT_NET_WORTH: f64 =
        CHECKING_BALANCE - FREEDOM_UNLIMITED_BALANCE - DISCOVER_BALANCE
            + HUNTINGTON_BALANCE
            + SOFI_BALANCE
            + WALLET_BALANCE;

    #[test]
    fn test_defaults_offset_calcs_offset_calc() {
        let yml = r#"
        checking-balance: 101.34
        freedom-unlimited-balance: 34.23
        discover-balance: 890.32
        huntington-balance: 123.34
        sofi-balance: 10345.23
        wallet-balance: 10.45
        "#;

        let offset_calc_task: OffsetCalcTask = serde_yaml::from_str(yml).unwrap();
        assert_eq!(offset_calc_task.checking_balance, 101.34);
        assert_eq!(offset_calc_task.freedom_unlimited_balance, 34.23);
        assert_eq!(offset_calc_task.discover_balance, 890.32);
        assert_eq!(offset_calc_task.huntington_balance, 123.34);
        assert_eq!(offset_calc_task.sofi_balance, 10345.23);
        assert_eq!(offset_calc_task.wallet_balance, 10.45);
        assert_eq!(offset_calc_task.sort, false);
    }

    #[test]
    fn test_from_cmd_valid_offset_calc() {
        let cmd = OffsetCalcCmd {
            checking_balance: 123.12,
            freedom_unlimited_balance: 23.42,
            discover_balance: 233.45,
            huntington_balance: 23.34,
            sofi_balance: 23.23,
            wallet_balance: 234.23,
            sort: true,
        };
        let cmd_dispatch = Command::OffsetCalc(cmd.clone());
        let task = OffsetCalcTask::from_cmd(cmd_dispatch).unwrap();
        assert_eq!(task.checking_balance, cmd.checking_balance);
        assert_eq!(
            task.freedom_unlimited_balance,
            cmd.freedom_unlimited_balance
        );
        assert_eq!(task.discover_balance, cmd.discover_balance);
        assert_eq!(task.huntington_balance, cmd.huntington_balance);
        assert_eq!(task.sofi_balance, cmd.sofi_balance);
        assert_eq!(task.wallet_balance, cmd.wallet_balance);
        assert_eq!(task.sort, cmd.sort)
    }

    #[test]
    fn test_from_cmd_invald_offset_calc() {
        assert!(
            OffsetCalcTask::from_cmd(Command::Sort)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("command sort not valid for task")
        )
    }

    #[test]
    fn test_run_no_offset_offset_calc() {
        let offset_calc_task = get_mock_offset_calc_task();

        let txn = Txn {
            id: Uuid::new_v4(),
            date: chrono::Local::now().date_naive(),
            description: "test".to_string(),
            amount: MOCK_CURRENT_NET_WORTH,
            category: Category::Deposit,
        };

        let mut execution_context = get_mock_execution_context(&txn);
        let init_len_txns = execution_context.get_txns().len();
        offset_calc_task.run(&mut execution_context).unwrap();
        assert_eq!(execution_context.get_txns().len(), init_len_txns);
    }

    #[test]
    fn test_run_positive_offset_offset_calc() {
        let offset = 2.00;

        let txn = Txn {
            id: Uuid::new_v4(),
            date: chrono::Local::now().date_naive(),
            description: "test".to_string(),
            amount: MOCK_CURRENT_NET_WORTH - offset,
            category: Category::Deposit,
        };

        let mut offset_calc_task = get_mock_offset_calc_task();
        offset_calc_task.sort = false;
        let mut execution_context = get_mock_execution_context(&txn);
        let init_len_txns = execution_context.get_txns().len();
        offset_calc_task.run(&mut execution_context).unwrap();
        let txns = execution_context.get_txns();
        assert!(txns.len() == init_len_txns + 1);

        // Txn should have been added at last index because no sorting occured
        let added_txn = txns.last().unwrap();
        assert_eq!(added_txn.description.as_str(), "offset");
        assert_eq!(added_txn.amount, 2.00);
        assert_eq!(added_txn.category, Category::Deposit)
    }

    #[test]
    fn test_run_negative_offset_offset_calc() {
        let offset = 2.00;

        // Caclulated is going to be more than actual
        // Reduce the calculated to the actual by adding a needs txn equal to the offset
        // The needs txns will subtract from the calculated and equalize the numbers
        let txn = Txn {
            id: Uuid::new_v4(),
            date: chrono::Local::now().date_naive(),
            description: "test".to_string(),
            amount: MOCK_CURRENT_NET_WORTH + offset,
            category: Category::Deposit,
        };

        let mut offset_calc_task = get_mock_offset_calc_task();
        offset_calc_task.sort = false;
        let mut execution_context = get_mock_execution_context(&txn);
        let init_len_txns = execution_context.get_txns().len();
        offset_calc_task.run(&mut execution_context).unwrap();

        let txns = execution_context.get_txns();
        assert!(txns.len() == init_len_txns + 1);

        // Offset should have been added last in txns since no sort occured
        let added_txn = txns.last().unwrap();
        assert_eq!(added_txn.description.as_str(), "offset");
        assert_eq!(added_txn.amount, 2.00);
        assert_eq!(added_txn.category, Category::Needs);

        //
    }

    // The offset is going to be added with todays date
    // Use a super old date to make sure we sort such that the offset is first
    #[test]
    fn test_offset_calc_sort_offset_calc() {
        // Note - we need an old date
        let txn = Txn {
            id: Uuid::new_v4(),
            date: chrono::NaiveDate::from_ymd_opt(1800, 1, 1).unwrap(), // Ignorantly old date to check sort - sort value txn will get todays date.
            description: "test".to_string(),
            amount: MOCK_CURRENT_NET_WORTH + 2.00,
            category: Category::Deposit,
        };

        let mut task = get_mock_offset_calc_task();
        task.sort = true;
        let mut execution_context = get_mock_execution_context(&txn);
        let init_len_txns = execution_context.get_txns().len();
        task.run(&mut execution_context).unwrap();

        let txns = execution_context.get_txns();
        assert!(txns.len() == init_len_txns + 1);
        assert_eq!(txns.first().unwrap().description.as_str(), "offset")
        //
    }

    fn get_mock_execution_context(txn: &Txn) -> ExecutionContext {
        let txns = Vec::from([txn.clone()]);
        let header = HEADER.iter().map(|&col| col.to_string()).collect();
        let txns = Txns::new(txns, header);

        ExecutionContext::new(txns, BudgetCfg::default(), None)
    }

    fn get_mock_offset_calc_task() -> OffsetCalcTask {
        OffsetCalcTask {
            checking_balance: CHECKING_BALANCE,
            freedom_unlimited_balance: FREEDOM_UNLIMITED_BALANCE,
            discover_balance: DISCOVER_BALANCE,
            huntington_balance: HUNTINGTON_BALANCE,
            sofi_balance: SOFI_BALANCE,
            wallet_balance: WALLET_BALANCE,
            sort: false,
        }
    }
}
