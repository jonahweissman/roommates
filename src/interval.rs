use chrono::naive::NaiveDate;

use super::roommate::Roommate;

pub struct ResponsibilityInterval {
    roommate: Roommate,
    responsible_for_count: u32,
    interval: DateInterval,
}

impl ResponsibilityInterval {
    pub fn new(
        roommate: Roommate,
        responsible_for_count: u32,
        interval: DateInterval,
    ) -> Self {
        ResponsibilityInterval {
            roommate,
            responsible_for_count,
            interval,
        }
    }

    pub fn start(&self) -> NaiveDate {
        self.interval.0
    }

    pub fn end(&self) -> NaiveDate {
        self.interval.1
    }

    pub fn roommate(&self) -> &Roommate {
        &self.roommate
    }

    pub fn responsible_for_count(&self) -> u32 {
        self.responsible_for_count
    }
}

#[derive(Debug, Clone)]
pub struct DateInterval(NaiveDate, NaiveDate);

impl DateInterval {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        assert!(start < end, "start of interval must be less than end");
        DateInterval(start, end)
    }

    pub fn from_strs(start: &str, end: &str) -> Self {
        let start = NaiveDate::parse_from_str(start, "%m/%d/%Y").expect("Invalid start date");
        let end = NaiveDate::parse_from_str(end, "%m/%d/%Y").expect("Invalid end date");
        DateInterval::new(start, end)
    }

    pub fn interval(&self) -> (NaiveDate, NaiveDate) {
        (self.0, self.1)
    }
}
