mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate;
use tempfile;

use crate::common::{get_tmp_file_str_path, init_filesystem};

#[test]
fn test_write_to_file() {
    let (budget_cfg_path, txn_db_path) = init_filesystem();

    let output_path = tempfile::NamedTempFile::with_suffix(".yml").unwrap();
    let output_path_str = output_path.path().to_str().unwrap();

    cargo_bin_cmd!("rusty_budget")
        .arg("-c")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("-t")
        .arg(get_tmp_file_str_path(&txn_db_path))
        .arg("get-current-worth")
        .arg("-o")
        .arg(output_path_str)
        .assert()
        .success();

    let contents = std::fs::read_to_string(output_path_str).unwrap();
    assert!(contents.to_lowercase().contains("current_worth"));
}

#[test]
fn test_print_to_console() {
    let (budget_cfg_path, txn_db_path) = init_filesystem();

    cargo_bin_cmd!("rusty_budget")
        .arg("-c")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("-t")
        .arg(get_tmp_file_str_path(&txn_db_path))
        .arg("get-current-worth")
        .assert()
        .success()
        .stdout(predicate::str::contains("Current worth"));
}
