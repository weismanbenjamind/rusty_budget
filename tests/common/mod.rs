use chrono::NaiveDate;
use std::io::Write;
use tempfile::NamedTempFile;

pub const MOCK_UUID_1: &str = "f7a7a52f-ecdd-48d0-b0d8-4f892d7e7b6b";
pub const MOCK_TXN_DATE_1: NaiveDate = NaiveDate::from_ymd_opt(2018, 01, 15).unwrap();
pub const MOCK_AMOUNT_1: f64 = 45.32;
pub const MOCK_UUID_2: &str = "51fbe065-735b-43da-8d0c-cb548f76db25";
pub const MOCK_TXN_DATE_2: NaiveDate = NaiveDate::from_ymd_opt(2020, 5, 10).unwrap();
pub const MOCK_HEADER: &str = "id,date,description,amount,category";
pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const MOCK_NEEDS_GOAL: f64 = 4514.24;

pub fn get_mock_txn_1() -> String {
    format!(
        "{},{},groceries,{},Needs",
        MOCK_UUID_1,
        MOCK_TXN_DATE_1.format(DATE_FORMAT).to_string(),
        MOCK_AMOUNT_1
    )
}

pub fn get_mock_txn_2() -> String {
    format!(
        "{},{},movie,15.99,Wants",
        MOCK_UUID_2,
        MOCK_TXN_DATE_2.format(DATE_FORMAT).to_string()
    )
}

pub fn get_mock_budget_cfg_contents() -> String {
    format!(
        "monthly_goals:\n  savings: 1023.54\n  needs: {}\n  wants: 809.28\n\nweekly_wants_goal: 150.23\n",
        MOCK_NEEDS_GOAL,
    )
}

pub fn init_filesystem() -> (NamedTempFile, NamedTempFile) {
    (init_tmp_budet_cfg(), init_tmp_txn_db())
}

pub fn init_tmp_budet_cfg() -> NamedTempFile {
    let mut temp_file = NamedTempFile::with_suffix(".yml").unwrap();
    write!(temp_file, "{}", get_mock_budget_cfg_contents()).unwrap();

    temp_file
}

pub fn init_tmp_txn_db() -> NamedTempFile {
    let mut temp_file = init_tmp_csv();
    write!(
        temp_file,
        "{}\n{}\n{}\n",
        MOCK_HEADER,
        get_mock_txn_1(),
        get_mock_txn_2()
    )
    .unwrap();

    temp_file
}

pub fn init_tmp_csv() -> NamedTempFile {
    NamedTempFile::with_suffix(".csv").unwrap()
}

pub fn get_tmp_file_str_path(temp_file: &NamedTempFile) -> &str {
    temp_file.path().to_str().unwrap()
}
