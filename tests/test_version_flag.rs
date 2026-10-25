use assert_cmd::cargo_bin_cmd;
use predicates::prelude::predicate;

const VERSION_REGEX: &'static str = r"\d+.\d+.\d+";

#[test]
fn test_version_flag_long() {
    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("--version");
    cmd.assert().success();

    cmd.assert()
        .stdout(predicate::str::is_match(VERSION_REGEX).unwrap());
}

#[test]
fn test_version_flag_short() {
    let mut cmd = cargo_bin_cmd!("rusty_budget");
    cmd.arg("-v");
    cmd.assert().success();
    cmd.assert().success();

    cmd.assert()
        .stdout(predicate::str::is_match(VERSION_REGEX).unwrap());
}
