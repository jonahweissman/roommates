use chrono::naive::NaiveDate;
use std::cmp::{max, min};
use std::iter::FromIterator;
use std::slice::Iter;

use super::roommate::Roommate;

pub struct ResponsibilityRecord {
    intervals: Vec<ResponsibilityInterval>,
}

impl ResponsibilityRecord {
    pub fn iter(&self) -> Iter<ResponsibilityInterval> {
        self.intervals.iter()
    }

    /// The occupancy represented by the responsibility intervals
    /// measured in `person x days` units.
    pub fn occupancy(&self, billing_period: DateInterval) -> u32 {
        let (start, end) = billing_period.interval();
        self.iter()
            .map(|i| {
                i.responsible_for_count()
                    * max(
                        0,
                        min(i.end(), end)
                            .signed_duration_since(max(i.start(), start))
                            .num_days(),
                    ) as u32
            })
            .sum()
    }
}

impl FromIterator<ResponsibilityInterval> for ResponsibilityRecord {
    fn from_iter<I: IntoIterator<Item = ResponsibilityInterval>>(iter: I) -> Self {
        let intervals = iter.into_iter().collect::<Vec<_>>();
        ResponsibilityRecord { intervals }
    }
}

#[derive(Clone)]
pub struct ResponsibilityInterval {
    roommate: Roommate,
    responsible_for_count: u32,
    interval: DateInterval,
}

impl ResponsibilityInterval {
    pub fn new(roommate: Roommate, responsible_for_count: u32, interval: DateInterval) -> Self {
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

#[derive(Debug, Clone, Copy)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveDate;

    #[test]
    fn whole_interval() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![ResponsibilityInterval::new(
            Roommate::new("me"),
            1,
            DateInterval::new(start, end),
        )]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(intervals.occupancy(DateInterval::new(start, end)), 31,);
    }

    #[test]
    fn partial_interval_with_weights() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            ResponsibilityInterval::new(
                Roommate::new("me"),
                2,
                DateInterval::new(NaiveDate::parse_from_str("01/18/20", "%D").unwrap(), end),
            ),
            ResponsibilityInterval::new(
                Roommate::new("someone"),
                4,
                DateInterval::new(start, NaiveDate::parse_from_str("01/13/20", "%D").unwrap()),
            ),
        ]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(
            intervals.occupancy(DateInterval::new(start, end)),
            4 * 3 + 2 * 2,
        );
    }

    #[test]
    fn capping_intervals() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            ResponsibilityInterval::new(
                Roommate::new("me"),
                2,
                DateInterval::new(
                    NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                    NaiveDate::parse_from_str("01/20/21", "%D").unwrap(),
                ),
            ),
            ResponsibilityInterval::new(
                Roommate::new("someone"),
                4,
                DateInterval::new(
                    NaiveDate::parse_from_str("01/10/19", "%D").unwrap(),
                    NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                ),
            ),
        ]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(
            intervals.occupancy(DateInterval::new(start, end)),
            4 * 3 + 2 * 2,
        );
    }
}
