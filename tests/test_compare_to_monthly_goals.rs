use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate;
use tempfile;

use crate::common::{get_tmp_file_str_path, init_filesystem};

mod common;

#[test]
fn test_compare_to_monthly_goals_print_to_console() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("compare-to-monthly-goals");

    cmd.assert().success().stdout(predicate::str::contains(
        "monthly_needs_spend (goal-actual):",
    ));
}

#[test]
fn test_compare_to_monthly_goals() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_tmp_file = tempfile::NamedTempFile::with_suffix(".yml").unwrap();
    let output_path = get_tmp_file_str_path(&output_tmp_file);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("compare-to-monthly-goals")
        .arg("--output-path")
        .arg(output_path);

    cmd.assert().success();
    let contents = std::fs::read_to_string(output_path).unwrap();
    assert!(contents.contains("monthly_needs_spend (goal-actual):"));
    assert!(contents.contains("monthly_wants_spend (goal-actual):"));
    assert!(contents.contains("monthly_savings (actual-goal):"));
    assert!(contents.contains("weekly_wants_spend (goal-actual):"))
}
