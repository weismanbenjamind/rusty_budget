use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::read_to_string;

mod common;
use common::{
    MOCK_AMOUNT_1, MOCK_HEADER, MOCK_UUID_1, get_mock_txn_1, get_mock_txn_2, get_tmp_file_str_path,
    init_filesystem,
};

#[test]
fn test_update_txn() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txn_db_path = get_tmp_file_str_path(&txns_db_path);
    let new_amount = MOCK_AMOUNT_1 + 54.47;

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txn_db_path)
        .arg("update-txn")
        .arg("--id")
        .arg(MOCK_UUID_1)
        .arg("--amount")
        .arg(&new_amount.to_string());

    cmd.assert().success();

    let output_csv = read_to_string(txn_db_path).unwrap();
    let expected_new_amount = (new_amount * 100.0).round() / 100.0;
    let expected_row =
        get_mock_txn_1().replace(&MOCK_AMOUNT_1.to_string(), &expected_new_amount.to_string());
    assert!(output_csv.contains(MOCK_HEADER));
    assert!(output_csv.contains(&expected_row));
    assert!(output_csv.contains(&get_mock_txn_2()));
}

#[test]
fn test_update_txn_sort() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txn_db_path = get_tmp_file_str_path(&txns_db_path);
    let new_amount = MOCK_AMOUNT_1 + 54.47;

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txn_db_path)
        .arg("update-txn")
        .arg("--id")
        .arg(MOCK_UUID_1)
        .arg("--amount")
        .arg(&new_amount.to_string())
        .arg("--sort");

    cmd.assert().success();
}
