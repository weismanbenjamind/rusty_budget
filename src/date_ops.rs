use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};

pub fn parse_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse date {}. Make sure date is in YYYY-MM-DD format. Error: {}",
            date_str,
            e
        )
    })
}

pub fn get_start_of_week(date: NaiveDate) -> NaiveDate {
    let mut start_of_week = date - chrono::Days::new(date.weekday().num_days_from_sunday().into());
    if start_of_week.month() != date.month() {
        start_of_week = get_first_of_month(date)
    }
    start_of_week
}

pub fn get_first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap()
}

pub fn get_days_left_this_month(date: NaiveDate) -> u32 {
    (date.num_days_in_month() as u32) - date.day() + 1 // Note - The +1 make this an inclusive calculation (e.g. Assuming a 31 day month: 31st day is one day left. 1st day is 31 days left.)
}

pub fn get_days_left_this_week(date: NaiveDate) -> u32 {
    7u32 - date.weekday().num_days_from_sunday() // Note - this is an inclusive operation (E.g. sunday has 7 days left in the week. Saturday has 1 day left in the week)
}

pub fn get_days_left_this_week_end_of_month_aware(date: NaiveDate) -> u32 {
    let days_left_this_month = get_days_left_this_month(date);
    let days_left_this_week = get_days_left_this_week(date);
    match days_left_this_month < days_left_this_week {
        true => days_left_this_month,
        false => days_left_this_week,
    }
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;

    #[parameterized(date_str = {"2025-12-29", "2025-1-29", "2025-01-29", "2024-11-1", "2024-11-01"})]
    fn test_parse_date_valid(date_str: &str) -> Result<()> {
        let split: Vec<&str> = date_str.split("-").collect();

        let year: i32 = split[0].parse()?;
        let month: u32 = split[1].parse()?;
        let day: u32 = split[2].parse()?;

        let target_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow::anyhow!("Failed to create target date"))?;

        let found_date = parse_date(date_str)?;

        assert_eq!(target_date, found_date);
        Ok(())
    }

    #[parameterized(date_str = {"Invalid", "12-29-2025", "1-29-2025", "12-1-2025"})]
    fn test_parse_date_error(date_str: &str) {
        let result = parse_date(date_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse date")
        );
    }

    #[test]
    fn test_get_start_of_week_sunday_today() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
        let found = get_start_of_week(date);
        assert_eq!(found, date);
    }

    #[test]
    fn test_get_start_of_week_last_sunday_prev_month() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        let found = get_start_of_week(date);
        let expected = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(found, expected);
    }

    #[test]
    fn test_get_start_of_week_last_sunday_curr_month() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 17).unwrap();
        let found = get_start_of_week(date);
        let expected = NaiveDate::from_ymd_opt(2026, 1, 11).unwrap();
        assert_eq!(found, expected);
    }

    #[test]
    fn test_get_first_of_month() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 13).unwrap();
        let found = get_first_of_month(date);
        assert!(found.day() == 1);
        assert!(found.year() == date.year());
        assert!(found.month() == date.month());
    }

    #[test]
    fn test_get_days_left_this_month() {
        // 31 days in January
        let mut today = chrono::NaiveDate::from_ymd_opt(2026, 1, 24).unwrap();
        assert_eq!(get_days_left_this_month(today), 8);

        // 31 days in January
        today = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(get_days_left_this_month(today), 31);

        // 31 days in January
        today = chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        assert_eq!(get_days_left_this_month(today), 1);
    }

    #[test]
    fn test_get_days_left_this_week() {
        // Saturday
        let mut today = chrono::NaiveDate::from_ymd_opt(2026, 1, 24).unwrap();
        assert_eq!(get_days_left_this_week(today), 1);

        // Sunday
        today = chrono::NaiveDate::from_ymd_opt(2026, 1, 25).unwrap();
        assert_eq!(get_days_left_this_week(today), 7);

        // Thursday
        today = chrono::NaiveDate::from_ymd_opt(2026, 1, 22).unwrap();
        assert_eq!(get_days_left_this_week(today), 3);
    }

    #[test]
    fn test_days_left_this_week_end_of_month_aware_month_ends_this_week() {
        // 2 Days left when today is monday
        let date = create_naive_date(2026, 3, 30);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 2);

        // 1 day left for last day of month where last day of month is a Wednesday
        let date = create_naive_date(2026, 3, 31);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 1);

        // 3 days left for last day of month where today is a Sunday
        let date = create_naive_date(2026, 3, 29);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 3);
    }

    #[test]
    fn test_days_left_this_week_end_of_month_aware_month_ends_following_week() {
        // March 26, 2026 -> End of week is 3 days away. End of month is 6 days away
        let date = create_naive_date(2026, 3, 26);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 3);

        // March 28, 2026 -> End of week is 1 day away (This is a Saturday). End of month is 4 days away
        let date = create_naive_date(2026, 3, 28);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 1);

        // March 25, 2026 -> End of week is 4 days away. End of month is 7 days away
        let date = create_naive_date(2026, 3, 25);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 4);
    }

    #[test]
    fn test_days_left_this_week_end_of_month_aware_last_week_of_month_7_days() {
        // Feb 22, 2026 -> End of month and week both 7 days away
        let date = create_naive_date(2026, 2, 22);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 7);

        // Feb 25, 2026 -> End of month and week both 4 days away
        let date = create_naive_date(2026, 2, 25);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 4);

        // Feb 28, 2026 -> End of month and week both 1 day away (not a leap year)
        let date = create_naive_date(2026, 2, 28);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 1);

        // Feb 21, 2026 -> End of month and week both 8 days away and end of week 1 day away (this is a saturday)
        let date = create_naive_date(2026, 2, 21);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 1);
    }

    #[test]
    fn test_days_left_this_week_end_of_month_aware_more_than_week_from_end_of_month() {
        let date = create_naive_date(2026, 3, 11);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 4);
    }

    #[test]
    fn test_days_left_this_week_end_of_month_aware_first_week_of_month() {
        let date = create_naive_date(2026, 3, 4);
        let days_left_this_week = get_days_left_this_week_end_of_month_aware(date);
        assert_eq!(days_left_this_week, 4);
    }

    fn create_naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }
}
