use crate::date_ops::parse_date;
use crate::txn_ops::Category;
use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};
use std::str::FromStr;
use uuid::Uuid;

fn serialize_amount<S: Serializer>(amount: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    let rounded = (amount * 100.0).round() / 100.0;
    serializer.serialize_f64(rounded)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Txn {
    pub id: Uuid,
    pub date: NaiveDate,
    pub description: String,

    #[serde(serialize_with = "serialize_amount")]
    pub amount: f64,

    pub category: Category,
}

impl Txn {
    pub fn from_csv_row(row: &csv::StringRecord) -> Result<Txn> {
        Ok(Txn {
            id: Uuid::parse_str(&row[0])?,
            date: parse_date(&row[1])?,
            description: row[2].to_string(),
            amount: row[3].parse()?,
            category: Category::from_str(&row[4])?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_csv_row() -> Result<()> {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let date = "2024-12-23";
        let description = "test";
        let amount = "23.46";
        let category = "needs";

        let record =
            csv::StringRecord::from(vec![uuid_str.as_str(), date, description, amount, category]);

        let txn = Txn::from_csv_row(&record)?;

        assert_eq!(uuid, txn.id);
        assert_eq!(NaiveDate::parse_from_str(date, "%Y-%m-%d")?, txn.date);
        assert_eq!(description, txn.description);
        assert_eq!(amount.parse::<f64>()?, txn.amount);
        assert_eq!(Category::Needs, txn.category);

        Ok(())
    }
}
