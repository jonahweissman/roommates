use chrono::naive::NaiveDate;
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::HashMap;

pub mod interval;
use super::roommate::Roommate;
use interval::OwnedInterval;

#[derive(Debug)]
pub struct NaiveDateInterval(NaiveDate, NaiveDate);

impl NaiveDateInterval {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        NaiveDateInterval(start, end)
    }
}

pub fn proportion_of_interval(
    intervals: &Vec<OwnedInterval>,
    billing_period: &NaiveDateInterval,
) -> f64 {
    let NaiveDateInterval(start, end) = billing_period;
    total_cost_in_interval(&intervals.iter().collect(), &billing_period) as f64
        / end.signed_duration_since(*start).num_days() as f64
}

fn total_cost_in_interval(
    intervals: &Vec<&OwnedInterval>,
    billing_period: &NaiveDateInterval,
) -> u32 {
    let NaiveDateInterval(start, end) = billing_period;
    intervals
        .iter()
        .map(|i| {
            i.weight
                * max(
                    0,
                    min(i.end, *end)
                        .signed_duration_since(max(i.start, *start))
                        .num_days(),
                ) as u32
        })
        .sum()
}

/// Returns the proportion of the total cost that each contributing party
/// is responsible for
pub fn individual_responsibilities(
    intervals: &Vec<OwnedInterval>,
    billing_period: &NaiveDateInterval,
) -> HashMap<Roommate, f64> {
    intervals
        .iter()
        .map(|i| &i.owner)
        .unique()
        .map(|roommate| {
            (
                roommate.clone(),
                proportion_by_roommate(intervals, &billing_period, roommate),
            )
        })
        .collect()
}

fn proportion_by_roommate(
    intervals: &Vec<OwnedInterval>,
    billing_period: &NaiveDateInterval,
    roommate: &Roommate,
) -> f64 {
    let val = total_cost_in_interval(
        &intervals.iter().filter(|i| &i.owner == roommate).collect(),
        billing_period,
    ) as f64
        / total_cost_in_interval(&intervals.iter().collect(), billing_period) as f64;
    if val.is_nan() {
        0.0
    } else {
        val
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn whole_interval() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![OwnedInterval {
            start,
            end,
            owner: Roommate::new(String::from("me")),
            weight: 1,
        }];
        assert_eq!(
            proportion_of_interval(&intervals, &NaiveDateInterval(start, end)),
            1.0
        );
    }

    #[test]
    fn partial_interval_with_weights() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            OwnedInterval {
                start: NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                end,
                owner: Roommate::new(String::from("me")),
                weight: 2,
            },
            OwnedInterval {
                start,
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner: Roommate::new(String::from("someone")),
                weight: 4,
            },
        ];
        let correct_proportion = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            proportion_of_interval(&intervals, &NaiveDateInterval(start, end)),
            correct_proportion
        );
    }

    #[test]
    fn capping_intervals() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            OwnedInterval {
                start: NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                end: NaiveDate::parse_from_str("01/20/21", "%D").unwrap(),
                owner: Roommate::new(String::from("me")),
                weight: 2,
            },
            OwnedInterval {
                start: NaiveDate::parse_from_str("01/10/19", "%D").unwrap(),
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner: Roommate::new(String::from("someone")),
                weight: 4,
            },
        ];
        let correct_proportion = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            proportion_of_interval(&intervals, &NaiveDateInterval(start, end)),
            correct_proportion
        );
    }
    #[test]
    fn partial_interval_with_weights_responsibilities() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            OwnedInterval {
                start: NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                end,
                owner: Roommate::new(String::from("me")),
                weight: 2,
            },
            OwnedInterval {
                start,
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner: Roommate::new(String::from("someone")),
                weight: 4,
            },
        ];
        let table = individual_responsibilities(&intervals, &NaiveDateInterval(start, end));
        let total = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            *table.get(&intervals[0].owner).unwrap(),
            (2.0 * 2.0) / 10.0 as f64 / total
        );
        assert_eq!(table.values().sum::<f64>(), 1.0);
    }

    #[test]
    fn no_overlap_between_billing_period_and_intervals() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![OwnedInterval {
            start: NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
            end: NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
            owner: Roommate::new(String::from("me")),
            weight: 1,
        }];
        let billing_period = NaiveDateInterval(start, end);
        assert_eq!(proportion_of_interval(&intervals, &billing_period), 0.0);
        assert_eq!(
            *individual_responsibilities(&intervals, &billing_period)
                .get(&intervals[0].owner)
                .unwrap(),
            0.0
        );
    }
}
