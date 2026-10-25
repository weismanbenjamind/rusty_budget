use crate::filesystem::generate_subdirs_for_output;
use crate::txn_ops::{Category, Txn};
use anyhow::Result;
use chrono::NaiveDate;
use csv::{self, Writer};
use log::info;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::{io, vec};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TxnsError {
    #[error("Failed to find txn with id {0}")]
    TxnNotFound(Uuid),

    #[error("Found multiple txns with id {0}")]
    MultipleTxns(Uuid),

    #[error("Txns in undefined state. Failed to find txn with id ")]
    UndefinedState(Uuid),

    #[error("Failed to write txns. Error {0}")]
    WriteError(String),
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Txns {
    txns: Vec<Txn>,
    header: Vec<String>,
}

impl Txns {
    pub fn get_header(&self) -> &Vec<String> {
        &self.header
    }

    pub fn get_all_txns(&self) -> &Vec<Txn> {
        &self.txns
    }

    #[allow(dead_code)]
    pub fn new(txns: Vec<Txn>, header: Vec<String>) -> Txns {
        Txns { txns, header }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.txns.len()
    }

    #[allow(dead_code)]
    pub fn last(&self) -> Option<&Txn> {
        self.txns.last()
    }

    #[allow(dead_code)]
    pub fn first(&self) -> Option<&Txn> {
        self.txns.first()
    }

    pub fn get_by_id_mut(&mut self, id: Uuid) -> Result<&mut Txn, TxnsError> {
        let mut txn_iter = self.txns.iter_mut().filter(|txn| txn.id == id);

        match (txn_iter.next(), txn_iter.next()) {
            (Some(txn), None) => Ok(txn),
            (None, None) => Err(TxnsError::TxnNotFound(id)),
            (Some(_), Some(_)) => Err(TxnsError::MultipleTxns(id)),
            (None, Some(_)) => Err(TxnsError::UndefinedState(id)), // In practice this should never get hit. Leaving it as a fallback
        }
    }

    pub fn from_csv(path: &str) -> Result<Txns> {
        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        let mut txns: Vec<Txn> = vec![];

        let header: Vec<String> = reader
            .headers()?
            .into_iter()
            .map(|record| record.to_string())
            .collect();

        for row in reader.records() {
            txns.push(Txn::from_csv_row(&row?)?)
        }
        Ok(Txns { txns, header })
    }

    pub fn add(
        &mut self,
        txn_date: NaiveDate,
        description: String,
        amount: f64,
        category: Category,
    ) {
        self.txns.push(Txn {
            id: get_new_uuid(self),
            date: txn_date,
            description,
            amount,
            category,
        })
    }

    pub fn get<'a>(&'a self, since: NaiveDate, until: NaiveDate, sort: bool) -> Vec<&'a Txn> {
        let mut subset: Vec<&'a Txn> = self
            .txns
            .iter()
            .filter(|&txn| txn.date >= since && txn.date <= until)
            .collect();

        if sort {
            subset.sort_by(|&a, &b| b.date.cmp(&a.date));
        }

        subset
    }

    pub fn get_wants_spend(&self, since: NaiveDate, until: NaiveDate) -> f64 {
        self.get(since, until, false)
            .into_iter()
            .filter(|&txn| txn.category == Category::Wants)
            .fold(0.0, |acc, txn| acc + txn.amount)
    }

    pub fn delete(&mut self, ids: &[Uuid]) {
        let unique_ids: HashSet<&Uuid> = ids.iter().collect();
        self.txns.retain(|txn| !unique_ids.contains(&txn.id));
    }

    pub fn get_current_worth(&self) -> f64 {
        self.txns.iter().fold(0.00, |acc, txn| match txn.category {
            Category::Deposit => acc + txn.amount,
            Category::Wants | Category::Needs => acc - txn.amount,
        })
    }

    pub fn write_to_csv(&mut self, path: &str, sort: bool) -> Result<()> {
        if sort {
            self.sort();
        }
        txns_to_csv(&self.txns, &self.header, path)
    }

    pub fn sort(&mut self) {
        self.txns.sort_by(|a, b| b.date.cmp(&a.date));
    }
}

pub fn txns_to_csv<T: Serialize, P: AsRef<Path>>(
    txns: &[T],
    header: &Vec<String>,
    path: P,
) -> Result<()> {
    let path = path.as_ref();
    generate_subdirs_for_output(path)?;

    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    info!("Writing txns to {}", path.display());
    serialize_to_csv_writer(txns, header, &mut writer)?;
    info!("Done");

    Ok(())
}

pub fn serialize_to_csv_writer<W: io::Write, T: Serialize>(
    txns: &[T],
    header: &Vec<String>,
    writer: &mut csv::Writer<W>,
) -> Result<(), TxnsError> {
    let write_result = match txns.is_empty() {
        true => writer.write_record(header),
        false => txns.iter().try_for_each(|txn| writer.serialize(txn)),
    };
    write_result.map_err(|err| TxnsError::WriteError(err.to_string()))?;
    writer
        .flush()
        .map_err(|err| TxnsError::WriteError(err.to_string()))
}

fn get_new_uuid(txns: &Txns) -> Uuid {
    let unique_ids: HashSet<&Uuid> = txns.txns.iter().map(|txn| &txn.id).collect();
    let mut new_id = Uuid::new_v4();
    loop {
        if !unique_ids.contains(&new_id) {
            return new_id;
        }
        new_id = Uuid::new_v4();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;
    use std::fs::{read_to_string, write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_new_uuid() {
        let txns = create_test_txns().0;
        let new_uuid = get_new_uuid(&txns);
        let original_uuids: Vec<Uuid> = txns.txns.iter().map(|txn| txn.id).collect();

        assert!(!original_uuids.contains(&new_uuid));
    }

    #[test]
    fn test_txns_to_csv_txns_from_csv_txns_exist() -> Result<()> {
        let (txns, expected_txns_csv) = create_test_txns();
        let (_tempfile, tempfile_path) = create_named_temp_file()?;

        txns_to_csv(&txns.txns, txns.get_header(), &tempfile_path)?;
        assert!(read_to_string(&tempfile_path)? == expected_txns_csv);

        Ok(())
    }

    #[test]
    fn test_txns_to_csv_txns_from_csv_txns_not_exist() -> Result<()> {
        let mut txns = create_test_txns().0;
        txns.txns = vec![];
        let (_tempfile, tempfile_path) = create_named_temp_file()?;

        txns_to_csv(&txns.txns, txns.get_header(), &tempfile_path)?;
        assert!(read_to_string(&tempfile_path)? == format!("{}\n", txns.header.join(",")));

        Ok(())
    }

    #[test]
    fn test_txns_to_csv() -> Result<()> {
        let (mut txns, expected_txns_csv) = create_test_txns();
        let (_tempfile, tempfile_path) = create_named_temp_file()?;

        txns.write_to_csv(&tempfile_path, false)?;
        assert!(read_to_string(&tempfile_path)? == expected_txns_csv);

        Ok(())
    }

    #[test]
    fn test_serialize_to_writer_only_header() {
        let mut buf = Vec::new();
        let mut writer = Writer::from_writer(&mut buf);

        let header_text = "header";
        let header = Vec::from([header_text.to_string()]);
        let txns: Vec<Txn> = Vec::new();

        serialize_to_csv_writer(&txns, &header, &mut writer).unwrap();
        drop(writer);

        let found = String::from_utf8(buf)
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
            .to_string();

        assert_eq!(found.as_str(), header_text)
    }

    #[test]
    fn test_serialize_to_writer_with_txn() {
        let header = vec!["id", "date", "description", "amount", "category"]
            .iter()
            .map(|&field| field.to_string())
            .collect();

        let id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(1, 2, 13).unwrap();
        let description = "test";
        let amount = 34.54;
        let category = Category::Wants;

        let txn = Txn {
            id,
            date,
            description: description.to_string(),
            amount,
            category,
        };
        let txns = vec![txn];

        let mut buf = Vec::new();
        let mut writer = Writer::from_writer(&mut buf);

        serialize_to_csv_writer(&txns, &header, &mut writer).unwrap();
        drop(writer);

        let found: Vec<String> = String::from_utf8(buf)
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].as_str(), header.join(","));
        assert_eq!(
            found[1],
            format!("{},{},{},{},{}", id, date, description, amount, category)
        )
    }

    #[test]
    fn test_txns_from_csv() {
        let (txns, expected_txns_csv) = create_test_txns();
        let (_tempfile, tempfile_path) = create_named_temp_file().unwrap();

        write(&tempfile_path, &expected_txns_csv).unwrap();

        assert!(Txns::from_csv(&tempfile_path).unwrap() == txns);
    }

    #[test]
    fn test_delete() -> Result<()> {
        let mut test_txns = create_test_txns().0;
        let id_to_delete = test_txns.txns.get(0).expect("Failed to get txn").id;
        let expected_txn = test_txns.txns.get(1).expect("Failed to get txn").clone();

        test_txns.delete(&[id_to_delete]);
        assert!(test_txns.txns.len() == 1);
        assert_eq!(
            test_txns.txns.get(0).expect("Failed to get txn"),
            &expected_txn
        );

        Ok(())
    }

    // Note - This test depends on on the dates returned from the txns in the create_text_txns function
    // If the txns returned from that function are updated this test may fail
    #[test]
    fn test_get_include_all_txns() {
        let test_txns = create_test_txns().0;
        let since = test_txns.txns.get(1).unwrap().date;
        let until = test_txns.txns.get(0).unwrap().date;

        let found_txns = test_txns.get(since, until, false);
        let expected_txns: Vec<&Txn> = test_txns.txns.iter().collect();
        assert_eq!(found_txns, expected_txns);
    }

    // Note - This test depends on on the dates returned from the txns in the create_text_txns function
    // If the txns returned from that function are updated this test may fail
    #[test]
    fn test_get_include_only_newest_txn() {
        let test_txns = create_test_txns().0;
        let since = test_txns.txns.get(1).unwrap().date + (Days::new(1));
        let target_date = test_txns.txns.get(0).unwrap().date;
        let until = target_date + (Days::new(1));

        let found_txns = test_txns.get(since, until, false);
        let expected_txns: Vec<&Txn> = test_txns
            .txns
            .iter()
            .filter(|&txn| txn.date == target_date)
            .collect();

        assert_eq!(found_txns, expected_txns);
    }

    // Note - This test depends on on the dates returned from the txns in the create_text_txns function
    // If the txns returned from that function are updated this test may fail
    #[test]
    fn test_get_include_only_oldest_txn() {
        let test_txns = create_test_txns().0;
        let target_date = test_txns.txns.get(1).unwrap().date;
        let since = target_date - Days::new(1);
        let until = test_txns.txns.get(0).unwrap().date - Days::new(1);

        let found_txns = test_txns.get(since, until, false);
        let expected_txns: Vec<&Txn> = test_txns
            .txns
            .iter()
            .filter(|&txn| txn.date == target_date)
            .collect();

        assert_eq!(found_txns, expected_txns);
    }

    // Note - This test depends on on the dates returned from the txns in the create_text_txns function
    // If the txns returned from that function are updated this test may fail
    #[test]
    fn test_get_include_no_txns() {
        let test_txns = create_test_txns().0;
        let since = test_txns.txns.get(1).unwrap().date + Days::new(1);
        let until = test_txns.txns.get(0).unwrap().date - Days::new(1);

        let found_txns = test_txns.get(since, until, false);
        assert!(found_txns.is_empty());
    }

    #[test]
    fn test_add() {
        let date = NaiveDate::from_ymd_opt(2023, 7, 16).unwrap();
        let description = "test".to_string();
        let amount = 550.89;
        let category = Category::Deposit;

        let mut test_txns = create_test_txns().0;

        test_txns.add(date, description.clone(), amount, category.clone());

        assert_eq!(test_txns.txns.len(), 3);

        let added_txn = test_txns.txns.last().unwrap();
        assert_eq!(added_txn.date, date);
        assert_eq!(added_txn.description, description);
        assert_eq!(added_txn.amount, amount);
        assert_eq!(added_txn.category, category)
    }

    #[test]
    fn test_get_header() {
        let test_txns = create_test_txns().0;
        assert_eq!(test_txns.header, *test_txns.get_header());
    }

    #[test]
    fn test_new() {
        let header: Vec<String> = ["id", "date", "description", "amount", "category"]
            .into_iter()
            .map(|ele| ele.to_string())
            .collect();

        let txn = Txn {
            id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            description: "test".to_string(),
            amount: 100.32,
            category: Category::Needs,
        };

        let txns = Txns::new(vec![txn.clone()], header.clone());
        assert_eq!(vec![txn], txns.txns);
        assert_eq!(header, txns.header);
    }

    #[test]
    fn testget_all_txns() {
        let test_txns = create_test_txns().0;
        assert_eq!(test_txns.txns, *test_txns.get_all_txns());
    }

    #[test]
    fn test_write_to_csv_sort() -> Result<()> {
        let description_1 = String::from("test");

        let txn_1 = Txn {
            id: Uuid::new_v4(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(),
            description: description_1.clone(),
            amount: 100.00,
            category: Category::Wants,
        };

        let description_2 = String::from("test_2");

        let txn_2 = Txn {
            id: Uuid::new_v4(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            description: description_2.clone(),
            amount: 100.00,
            category: Category::Wants,
        };

        let mut txns = Txns {
            txns: vec![txn_1, txn_2],
            header: vec!["header".to_string()],
        };

        let temp_file = NamedTempFile::with_suffix(".csv")?;
        let output_path = temp_file.path().to_str().unwrap();
        txns.write_to_csv(output_path, true)?;

        let found: Vec<String> = std::fs::read_to_string(output_path)?
            .strip_suffix("\n")
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();

        assert!(found.len() == 3);
        assert!(found.get(1).unwrap().contains(&description_2));
        assert!(found.get(2).unwrap().contains(&description_1));

        Ok(())
    }

    #[test]
    fn test_get_sort() -> Result<()> {
        let date_1 = chrono::NaiveDate::from_ymd_opt(2026, 1, 12).unwrap();

        let txn_1 = Txn {
            id: Uuid::new_v4(),
            date: date_1,
            description: String::from("test"),
            amount: 100.00,
            category: Category::Wants,
        };

        let date_2 = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        let txn_2 = Txn {
            id: Uuid::new_v4(),
            date: date_2,
            description: String::from("test_2"),
            amount: 100.00,
            category: Category::Wants,
        };

        let txns = Txns {
            txns: vec![txn_1.clone(), txn_2.clone()],
            header: vec!["header".to_string()],
        };

        let found = txns.get(date_1, date_2, true);
        assert_eq!(found.len(), 2);
        assert_eq!(*found.get(0).unwrap(), &txn_2);
        assert_eq!(*found.get(1).unwrap(), &txn_1);

        Ok(())
    }

    fn create_test_txns() -> (Txns, String) {
        let id_1 = Uuid::new_v4();
        let date_1 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let description_1 = "test".to_string();
        let amount_1 = 100.32;
        let category_1 = Category::Needs;

        let mut id_2 = Uuid::new_v4();
        loop {
            if id_2 != id_1 {
                break;
            }
            id_2 = Uuid::new_v4();
        }

        let date_2 = NaiveDate::from_ymd_opt(2024, 04, 16).unwrap();
        let description_2 = "test".to_string();
        let amount_2 = 63.37;
        let category_2 = Category::Wants;

        let header = ["id", "date", "description", "amount", "category"];
        // .into_iter()
        // .map(|ele| ele.to_string())
        // .collect();

        let txn_1 = Txn {
            id: id_1,
            date: date_1,
            description: description_1.clone(),
            amount: amount_1,
            category: category_1.clone(),
        };

        let txn_2 = Txn {
            id: id_2,
            date: date_2,
            description: description_2.clone(),
            amount: amount_2,
            category: category_2.clone(),
        };

        let header_as_csv_row = header.join(",");
        let txn_1_as_csv_row = txn_to_csv_row(id_1, date_1, &description_1, amount_1, category_1);
        let txn_2_as_csv_row = txn_to_csv_row(id_2, date_2, &description_2, amount_2, category_2);
        let expected_csv = format!(
            "{}\n{}\n{}\n",
            header_as_csv_row, txn_1_as_csv_row, txn_2_as_csv_row
        );

        let txns = Txns {
            txns: vec![txn_1, txn_2],
            header: header.into_iter().map(|s| s.to_string()).collect(),
        };

        (txns, expected_csv)
    }

    #[test]
    fn test_get_worth() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 01, 31).unwrap();
        let description = String::from("test");

        let deposit_amount = 100.0;
        let txn_1 = Txn {
            id: uuid::Uuid::new_v4(),
            date: date,
            description: description.clone(),
            amount: deposit_amount,
            category: Category::Deposit,
        };

        let wants_amount = 25.00;
        let txn_2 = Txn {
            description: description.clone(),
            amount: wants_amount,
            category: Category::Wants,
            ..txn_1
        };

        let needs_amount = 5.00;
        let txn_3 = Txn {
            description: description.clone(),
            amount: needs_amount,
            category: Category::Needs,
            ..txn_1
        };

        let txns = Txns {
            txns: vec![txn_1, txn_2, txn_3],
            header: vec![String::from("Headers")],
        };

        let found_current_worth = txns.get_current_worth();
        let expected_current_worth = deposit_amount - wants_amount - needs_amount;

        assert_eq!(found_current_worth, expected_current_worth);
    }

    #[test]
    fn test_first_and_last_txns() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 01, 31).unwrap();
        let description = String::from("test");

        let deposit_amount = 100.0;
        let txn_1 = Txn {
            id: uuid::Uuid::new_v4(),
            date: date,
            description: description.clone(),
            amount: deposit_amount,
            category: Category::Deposit,
        };

        let wants_amount = 25.00;
        let txn_2 = Txn {
            description: description.clone(),
            amount: wants_amount,
            category: Category::Wants,
            ..txn_1
        };

        let needs_amount = 5.00;
        let txn_3 = Txn {
            description: description.clone(),
            amount: needs_amount,
            category: Category::Needs,
            ..txn_1
        };

        let txns = Txns {
            txns: vec![txn_1.clone(), txn_2, txn_3.clone()],
            header: vec![String::from("Headers")],
        };

        assert_eq!(*txns.first().unwrap(), txn_1);
        assert_eq!(*txns.last().unwrap(), txn_3);
    }

    #[test]
    fn test_first_and_last_empty_txns() {
        let txns = Txns::default();
        assert_eq!(txns.first(), None);
        assert_eq!(txns.last(), None);
    }

    #[test]
    fn test_txns_get_by_id_mut_valid() {
        let id_1 = Uuid::new_v4();
        let id_2 = Uuid::new_v4();
        assert!(id_1 != id_2);

        let txn_1 = Txn {
            id: id_1,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 100.23,
            category: Category::Wants,
        };

        let txn_2 = Txn {
            id: id_2,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 100.23,
            category: Category::Wants,
        };

        let mut txns = Txns {
            txns: vec![txn_1.clone(), txn_2],
            header: get_dummy_header(),
        };

        let found = txns.get_by_id_mut(id_1).unwrap();
        assert_eq!(txn_1, *found);
    }

    #[test]
    fn test_txns_get_by_id_valid_single_txn() {
        let id_1 = Uuid::new_v4();

        let txn_1 = Txn {
            id: id_1,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 100.23,
            category: Category::Wants,
        };

        let mut txns = Txns {
            txns: vec![txn_1.clone()],
            header: get_dummy_header(),
        };

        let found = txns.get_by_id_mut(id_1).unwrap();
        assert_eq!(txn_1, *found);
    }

    #[test]
    fn test_txns_get_by_id_txn_not_found() {
        let id_1 = Uuid::new_v4();
        let id_2 = Uuid::new_v4();
        assert_ne!(id_1, id_2);

        let txn_1 = Txn {
            id: id_1,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 100.23,
            category: Category::Wants,
        };

        let mut txns = Txns {
            txns: vec![txn_1],
            header: get_dummy_header(),
        };

        assert!(
            txns.get_by_id_mut(id_2)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("failed to find txn")
        );
    }

    #[test]
    fn test_txns_get_by_id_multi_id() {
        let id = Uuid::new_v4();

        let txn_1 = Txn {
            id: id,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 100.23,
            category: Category::Wants,
        };

        let txn_2 = Txn {
            id: id,
            date: get_dummy_naive_date(),
            description: get_dummy_desc(),
            amount: 24.23,
            category: Category::Needs,
        };

        let mut txns = Txns {
            txns: vec![txn_1, txn_2],
            header: get_dummy_header(),
        };

        let err = txns.get_by_id_mut(id).unwrap_err();
        match err {
            TxnsError::MultipleTxns(found_id) => assert_eq!(id, found_id),
            _ => panic!("Did not find multiple txns error. Found {:.?}", err),
        }
    }

    fn get_dummy_naive_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(1, 1, 15).unwrap()
    }

    fn get_dummy_desc() -> String {
        "some".to_string()
    }

    fn get_dummy_header() -> Vec<String> {
        vec!["".to_string()]
    }

    fn txn_to_csv_row(
        id: Uuid,
        date: NaiveDate,
        description: &str,
        amount: f64,
        category: Category,
    ) -> String {
        format!(
            "{},{},{},{},{}",
            id,
            date.format("%Y-%m-%d").to_string(),
            description,
            amount,
            category
        )
    }

    fn create_named_temp_file() -> Result<(NamedTempFile, String)> {
        let temp_file = NamedTempFile::with_suffix(".csv")?;
        let file_path = temp_file.path().to_str().unwrap().to_string();
        Ok((temp_file, file_path))
    }
}
