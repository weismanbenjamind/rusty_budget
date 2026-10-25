use assert_cmd::cargo::cargo_bin_cmd;

mod common;

use common::init_filesystem;

use crate::common::get_tmp_file_str_path;

#[test]
fn test_offset_calc() {
    let (budget_cfg_path, txn_db_path) = init_filesystem();
    let txn_db_path = get_tmp_file_str_path(&txn_db_path);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txn_db_path)
        .arg("offset-calc")
        .arg("--checking-balance")
        .arg("10")
        .arg("--huntington-balance")
        .arg("10")
        .arg("--sofi-balance")
        .arg("10")
        .arg("--wallet-balance")
        .arg("10")
        .arg("--freedom-unlimited-balance")
        .arg("1")
        .arg("--discover-balance")
        .arg("1");

    cmd.assert().success();
    let found_contents = std::fs::read_to_string(txn_db_path).unwrap();
    assert_eq!(found_contents.matches(",offset,").count(), 1);
}

#[test]
fn test_offset_calc_sort() {
    let (budget_cfg_path, txn_db_path) = init_filesystem();
    let txn_db_path = get_tmp_file_str_path(&txn_db_path);

    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--cfg")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("--txn-db")
        .arg(txn_db_path)
        .arg("offset-calc")
        .arg("--checking-balance")
        .arg("10")
        .arg("--huntington-balance")
        .arg("10")
        .arg("--sofi-balance")
        .arg("10")
        .arg("--wallet-balance")
        .arg("10")
        .arg("--freedom-unlimited-balance")
        .arg("1")
        .arg("--discover-balance")
        .arg("1")
        .arg("-s");

    cmd.assert().success();
}
