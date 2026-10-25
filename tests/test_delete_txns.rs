use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::read_to_string;

mod common;
use common::{
    MOCK_HEADER, MOCK_UUID_1, MOCK_UUID_2, get_mock_txn_1, get_mock_txn_2, get_tmp_file_str_path,
    init_filesystem,
};

#[test]
fn test_delete_txns_single_txn_found() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("delete-txns")
        .arg("--ids")
        .arg(MOCK_UUID_1);

    cmd.assert().success();

    let contents = read_to_string(txns_db_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(!contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_delete_txns_all_txns_found() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("delete-txns")
        .arg("--ids")
        .arg(format!("{} {}", MOCK_UUID_1, MOCK_UUID_2));

    cmd.assert().success();

    let contents = read_to_string(txns_db_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(!contents.contains(&get_mock_txn_1()));
    assert!(!contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_delete_txns_txn_not_found() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let id_to_delete = "e7a7a52f-ecdd-48d0-b0d8-4f892d7e7b6b";
    assert!(id_to_delete != MOCK_UUID_1);
    assert!(id_to_delete != MOCK_UUID_2);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("delete-txns")
        .arg("--ids")
        .arg(id_to_delete);

    cmd.assert().success();

    let contents = read_to_string(txns_db_path).unwrap();
    assert!(contents.contains(MOCK_HEADER));
    assert!(contents.contains(&get_mock_txn_1()));
    assert!(contents.contains(&get_mock_txn_2()));
}

#[test]
fn test_delete_txns_sort() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();
    let txns_db_path = get_tmp_file_str_path(&txns_db_path);

    let id_to_delete = "e7a7a52f-ecdd-48d0-b0d8-4f892d7e7b6b";
    assert!(id_to_delete != MOCK_UUID_1);
    assert!(id_to_delete != MOCK_UUID_2);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txns_db_path)
        .arg("delete-txns")
        .arg("--ids")
        .arg(id_to_delete)
        .arg("-s");

    cmd.assert().success();
}
