mod common;

use assert_cmd::cargo::cargo_bin_cmd;

use crate::common::{get_tmp_file_str_path, init_filesystem};

#[test]
fn test_sort() {
    let (budget_cfg_path, txns_db_path) = init_filesystem();

    cargo_bin_cmd!("rusty_budget")
        .arg("-c")
        .arg(get_tmp_file_str_path(&budget_cfg_path))
        .arg("-t")
        .arg(get_tmp_file_str_path(&txns_db_path))
        .arg("sort")
        .assert()
        .success();
}
