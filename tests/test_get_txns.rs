use assert_cmd::cargo::cargo_bin_cmd;
use chrono::NaiveDate;
use predicates::prelude::predicate;
use std::fs::read_to_string;

mod common;
use common::{
    DATE_FORMAT, MOCK_HEADER, MOCK_TXN_DATE_1, MOCK_TXN_DATE_2, get_mock_txn_1, get_mock_txn_2,
    get_tmp_file_str_path, init_filesystem, init_tmp_csv,
};

#[test]
fn test_get_txns_get_all() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);
    let (until, since) = get_until_and_since();

    let date_offset = chrono::Days::new(1);
    let until = get_date_string(until + date_offset);
    let since = get_date_string(since - date_offset);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--until")
        .arg(until)
        .arg("--since")
        .arg(since.as_str())
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert().success();

    let contents = read_to_string(output_file_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_get_txns_since_and_until_equal() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);
    let (until, since) = get_until_and_since();

    let until = get_date_string(until);
    let since = get_date_string(since);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--until")
        .arg(until)
        .arg("--since")
        .arg(since.as_str())
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert().success();

    let contents = read_to_string(output_file_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_get_txns_get_none() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);
    let since = get_since();

    let until = get_date_string(since - chrono::Days::new(1));
    let since = get_date_string(since - chrono::Days::new(2));

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--until")
        .arg(until.as_str())
        .arg("--since")
        .arg(since.as_str())
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert().success();

    let contents = read_to_string(output_file_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(!contents.contains(&get_mock_txn_1()));
    assert!(!contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_get_txns_only_since() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);

    let since = get_date_string(get_since() - chrono::Days::new(1));

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--since")
        .arg(since.as_str())
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert().success();

    let contents = read_to_string(output_file_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_get_txns_only_one_txn() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);

    assert!(MOCK_TXN_DATE_1 != MOCK_TXN_DATE_2);
    let date = get_date_string(MOCK_TXN_DATE_1);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--since")
        .arg(date.as_str())
        .arg("--until")
        .arg(date)
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert().success();

    let contents = read_to_string(output_file_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(!contents.contains(&get_mock_txn_2()))
}

#[test]
fn test_get_txns_error_invalid_date() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--since")
        .arg("01-02-2024")
        .arg("--output-path")
        .arg(output_file_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse date"));
}

#[test]
fn test_get_txns_sort() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let output_file_path = init_tmp_csv();
    let output_file_path = get_tmp_file_str_path(&output_file_path);

    assert!(MOCK_TXN_DATE_1 != MOCK_TXN_DATE_2);
    let date = get_date_string(MOCK_TXN_DATE_1);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("get-txns")
        .arg("--since")
        .arg(date.as_str())
        .arg("--until")
        .arg(date)
        .arg("--output-path")
        .arg(output_file_path)
        .arg("--sort");

    cmd.assert().success();
}

fn get_since() -> NaiveDate {
    get_until_and_since().1
}

fn get_until_and_since() -> (NaiveDate, NaiveDate) {
    assert!(MOCK_TXN_DATE_1 != MOCK_TXN_DATE_2);
    match MOCK_TXN_DATE_1 > MOCK_TXN_DATE_2 {
        true => (MOCK_TXN_DATE_1, MOCK_TXN_DATE_2),
        false => (MOCK_TXN_DATE_2, MOCK_TXN_DATE_1),
    }
}

fn get_date_string(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}
