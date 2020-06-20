use chrono::naive::NaiveDate;

use super::super::roommate::Roommate;

pub struct ResponsibilityInterval {
    roommate: Roommate,
    responsible_for_count: u32,
    interval: (NaiveDate, NaiveDate),
}

impl ResponsibilityInterval {
    pub fn new(
        roommate: Roommate,
        responsible_for_count: u32,
        interval: (NaiveDate, NaiveDate)
    ) -> Self {
        assert!(interval.0 < interval.1, "start of interval must be less than end");
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

#[derive(Debug)]
pub struct NaiveDateInterval(NaiveDate, NaiveDate);

impl NaiveDateInterval {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        NaiveDateInterval(start, end)
    }

    pub fn interval(&self) -> (NaiveDate, NaiveDate) {
        (self.0, self.1)
    }
}
