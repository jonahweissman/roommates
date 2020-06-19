use chrono::naive::NaiveDate;
use csv::StringRecord;

use crate::roommate::Roommate;

pub struct OwnedInterval {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub owner: Roommate,
    pub weight: u32,
}

impl OwnedInterval {
    pub fn from_string_record(sr: StringRecord) -> Self {
        assert_eq!(sr.len(), 4, "Found row with wrong number of columns");
        let start = NaiveDate::parse_from_str(sr.get(2).expect("Missing start date"), "%m/%d/%Y")
            .expect("Invalid start date");
        let end = NaiveDate::parse_from_str(sr.get(3).expect("Missing end date"), "%m/%d/%Y")
            .expect("Invalid end date");
        assert!(start < end, "start of interval must be less than end");
        let owner = Roommate::new(String::from(sr.get(0).expect("Missing person")));
        let weight = sr
            .get(1)
            .expect("Missing count")
            .parse::<u32>()
            .expect("Invalid count");
        OwnedInterval {
            start,
            end,
            owner,
            weight,
        }
    }
}
