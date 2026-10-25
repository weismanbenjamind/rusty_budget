mod common;
use std::fs::read_to_string;

use assert_cmd::cargo::cargo_bin_cmd;
use common::{MOCK_NEEDS_GOAL, get_tmp_file_str_path, init_filesystem};
use predicates::prelude::predicate;

use crate::common::get_mock_budget_cfg_contents;

#[test]
fn test_update_budget_cfg() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let budget_cfg_path = get_tmp_file_str_path(&budget_cfg_path);
    let updated_needs_goal = format!("{:.2}", (MOCK_NEEDS_GOAL + 23.23));

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(budget_cfg_path)
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("update-budget-cfg")
        .arg("--needs-goal")
        .arg(&updated_needs_goal);

    cmd.assert().success();

    let expected_contents =
        get_mock_budget_cfg_contents().replace(&MOCK_NEEDS_GOAL.to_string(), &updated_needs_goal);
    let found_contents = read_to_string(budget_cfg_path).unwrap();

    assert_eq!(found_contents, expected_contents);
}

#[test]
fn test_auto_update_weekly_wants_goal() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("-c")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("-t")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("update-budget-cfg")
        .arg("--auto-update-weekly-wants-goal");

    cmd.assert().success();
}

#[test]
fn test_update_budget_cfg_manual_and_auto_pass_update_weekly_wants_spend() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("-c")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("-t")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("update-budget-cfg")
        .arg("--weekly-wants-goal")
        .arg("100.23")
        .arg("-a");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
