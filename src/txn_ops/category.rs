use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum CategoryError {
    Base(String),
}

impl std::fmt::Display for CategoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // * Note - match in future if have more than one variant
        let CategoryError::Base(err_msg) = self;
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for CategoryError {}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum Category {
    Needs,
    Wants,
    Deposit,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Category::Needs => write!(f, "Needs"),
            Category::Wants => write!(f, "Wants"),
            Category::Deposit => write!(f, "Savings"),
        }
    }
}

impl std::str::FromStr for Category {
    type Err = CategoryError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "needs" => Ok(Self::Needs),
            "wants" => Ok(Self::Wants),
            "deposit" => Ok(Self::Deposit),
            _ => Err(CategoryError::Base(format!("Invalid category {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;
    use std::str::FromStr;

    #[parameterized(category_str = {"needs", "wants", "deposit"}, expected = {Category::Needs, Category::Wants, Category::Deposit})]
    fn test_from_str_no_err(category_str: &str, expected: Category) -> Result<(), CategoryError> {
        assert_eq!(Category::from_str(category_str)?, expected);

        Ok(())
    }

    #[test]
    fn test_from_str_err() {
        let result = Category::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid category"))
    }
}
