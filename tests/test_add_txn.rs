use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate;
use std::fs::read_to_string;

mod common;
use common::{
    DATE_FORMAT, MOCK_HEADER, get_mock_txn_1, get_mock_txn_2, get_tmp_file_str_path,
    init_filesystem,
};

#[test]
fn test_add_txn_provide_date() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let date = "2024-08-09";
    let description = "test";
    let amount = "63.04";
    let category = "Needs";

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("add-txn")
        .arg("--date")
        .arg(date)
        .arg("--description")
        .arg(description)
        .arg("--amount")
        .arg(amount)
        .arg("--category")
        .arg(category);

    cmd.assert().success();

    let contents = read_to_string(txns_db_path).unwrap();
    let target_str = format!(",{},{},{},{}", date, description, amount, category);
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
    assert!(contents.contains(&target_str));
}

#[test]
fn test_add_txn_dont_provide_date() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let description = "test";
    let amount = "63.04";
    let category = "Needs";

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("add-txn")
        .arg("--description")
        .arg(description)
        .arg("--amount")
        .arg(amount)
        .arg("--category")
        .arg(category);

    cmd.assert().success();

    let contents = read_to_string(txns_db_path).unwrap();
    let expected_date = chrono::Local::now()
        .date_naive()
        .format(DATE_FORMAT)
        .to_string();
    let target_str = format!(",{},{},{},{}", expected_date, description, amount, category);
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
    assert!(contents.contains(&target_str));
}

#[test]
fn test_add_txn_invalid_category() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    let date = "2024-08-09";
    let description = "test";
    let amount = "63.04";
    let category = "invalid";

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("add-txn")
        .arg("--date")
        .arg(date)
        .arg("--description")
        .arg(description)
        .arg("--amount")
        .arg(amount)
        .arg("--category")
        .arg(category);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid category"));
}

#[test]
fn test_add_txn_sort() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("add-txn")
        .arg("--date")
        .arg("2024-08-09")
        .arg("--description")
        .arg("test")
        .arg("--amount")
        .arg("63.04")
        .arg("--category")
        .arg("wants")
        .arg("--sort");

    cmd.assert().success();
}
