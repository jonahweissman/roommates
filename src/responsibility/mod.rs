use itertools::Itertools;
use num::rational::Rational;
use std::cmp::{max, min};
use std::collections::HashMap;

use super::roommate::Roommate;
pub mod interval;
use interval::{DateInterval, ResponsibilityInterval};

pub fn proportion_of_interval(
    intervals: &Vec<ResponsibilityInterval>,
    billing_period: &DateInterval,
) -> Rational {
    let (start, end) = billing_period.interval();
    Rational::new(
        total_cost_in_interval(&intervals.iter().collect(), &billing_period) as isize,
        end.signed_duration_since(start).num_days() as isize,
    )
}

fn total_cost_in_interval(
    intervals: &Vec<&ResponsibilityInterval>,
    billing_period: &DateInterval,
) -> u32 {
    let (start, end) = billing_period.interval();
    intervals
        .iter()
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

/// Returns the proportion of the total cost that each contributing party
/// is responsible for
pub fn individual_responsibilities(
    intervals: &Vec<ResponsibilityInterval>,
    billing_period: &DateInterval,
) -> HashMap<Roommate, Rational> {
    intervals
        .iter()
        .map(|i| i.roommate())
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
    intervals: &Vec<ResponsibilityInterval>,
    billing_period: &DateInterval,
    roommate: &Roommate,
) -> Rational {
    let total_cost = total_cost_in_interval(&intervals.iter().collect(), billing_period);
    if total_cost == 0 {
        return Rational::from_integer(0);
    }
    Rational::new(
        total_cost_in_interval(
            &intervals
                .iter()
                .filter(|i| i.roommate() == roommate)
                .collect(),
            billing_period,
        ) as isize,
        total_cost as isize,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::naive::NaiveDate;

    #[test]
    fn whole_interval() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![ResponsibilityInterval::new(
            Roommate::new(String::from("me")),
            1,
            (start, end),
        )];
        assert_eq!(
            proportion_of_interval(&intervals, &DateInterval::new(start, end)),
            Rational::new(1 as isize, 1 as isize)
        );
    }

    #[test]
    fn partial_interval_with_weights() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            ResponsibilityInterval::new(
                Roommate::new(String::from("me")),
                2,
                (NaiveDate::parse_from_str("01/18/20", "%D").unwrap(), end),
            ),
            ResponsibilityInterval::new(
                Roommate::new(String::from("someone")),
                4,
                (start, NaiveDate::parse_from_str("01/13/20", "%D").unwrap()),
            ),
        ];
        let correct_proportion = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            proportion_of_interval(&intervals, &DateInterval::new(start, end)),
            Rational::approximate_float(correct_proportion).unwrap(),
        );
    }

    #[test]
    fn capping_intervals() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            ResponsibilityInterval::new(
                Roommate::new(String::from("me")),
                2,
                (
                    NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                    NaiveDate::parse_from_str("01/20/21", "%D").unwrap(),
                ),
            ),
            ResponsibilityInterval::new(
                Roommate::new(String::from("someone")),
                4,
                (
                    NaiveDate::parse_from_str("01/10/19", "%D").unwrap(),
                    NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                ),
            ),
        ];
        let correct_proportion = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            proportion_of_interval(&intervals, &DateInterval::new(start, end)),
            Rational::approximate_float(correct_proportion).unwrap()
        );
    }
    #[test]
    fn partial_interval_with_weights_responsibilities() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![
            ResponsibilityInterval::new(
                Roommate::new(String::from("me")),
                2,
                (NaiveDate::parse_from_str("01/18/20", "%D").unwrap(), end),
            ),
            ResponsibilityInterval::new(
                Roommate::new(String::from("someone")),
                4,
                (start, NaiveDate::parse_from_str("01/13/20", "%D").unwrap()),
            ),
        ];
        let table = individual_responsibilities(&intervals, &DateInterval::new(start, end));
        let total = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(
            *table.get(&intervals[0].roommate()).unwrap(),
            Rational::approximate_float((2.0 * 2.0) / 10.0 as f64 / total).unwrap()
        );
        assert_eq!(
            table.values().fold(Rational::from_integer(0), |a, x| a + x),
            Rational::from_integer(1),
        );
    }

    #[test]
    fn no_overlap_between_billing_period_and_intervals() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![ResponsibilityInterval::new(
            Roommate::new(String::from("me")),
            1,
            (
                NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
                NaiveDate::parse_from_str("02/02/19", "%D").unwrap(),
            ),
        )];
        let billing_period = DateInterval::new(start, end);
        assert_eq!(
            proportion_of_interval(&intervals, &billing_period),
            Rational::from_integer(0)
        );
        assert_eq!(
            *individual_responsibilities(&intervals, &billing_period)
                .get(&intervals[0].roommate())
                .unwrap(),
            Rational::from_integer(0)
        );
    }
}
