use itertools::Itertools;
use num::rational::Ratio;
use std::cmp::{max, min};
use std::collections::HashMap;

use super::interval::{DateInterval, ResponsibilityInterval};
use super::roommate::{Roommate, RoommateGroup};
pub mod split;
use split::ResponsibilitySplit;

impl DateInterval {
    /// The occupancy represented by the responsibility intervals
    /// measured in `person x days` units.
    pub fn occupancy<'a, I>(&self, intervals: I) -> u32
    where
        I: Iterator<Item = &'a ResponsibilityInterval>,
    {
        let (start, end) = self.interval();
        intervals
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

    fn roommate_responsibility<'a, I>(&self, roommate: &Roommate, intervals: I) -> Ratio<u32>
    where
        I: Iterator<Item = &'a ResponsibilityInterval>,
    {
        let (intervals, roommate_intervals) = intervals.tee();
        let roommate_intervals = roommate_intervals.filter(|i| i.roommate() == roommate);
        let total_cost = self.occupancy(intervals);
        if total_cost == 0 {
            return Ratio::from_integer(0);
        }
        Ratio::new(self.occupancy(roommate_intervals), total_cost)
    }
}

impl RoommateGroup {
    /// Returns the proportion of the total cost that each contributing party
    /// is responsible for
    pub fn individual_responsibilities<'a, I>(
        &self,
        intervals: I,
        billing_period: DateInterval,
    ) -> ResponsibilitySplit
    where
        I: IntoIterator<Item = &'a ResponsibilityInterval>,
    {
        let intervals = intervals.into_iter().collect::<Vec<_>>();
        let map: HashMap<Roommate, Ratio<u32>> = self
            .set()
            .iter()
            .map(|roommate| {
                (
                    roommate.clone(),
                    billing_period.roommate_responsibility(roommate, intervals.iter().copied()),
                )
            })
            .collect();
        self.build_split(map)
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
        )];
        assert_eq!(
            DateInterval::new(start, end).occupancy(intervals.iter()),
            31,
        );
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
        ];
        assert_eq!(
            DateInterval::new(start, end).occupancy(intervals.iter()),
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
        ];
        assert_eq!(
            DateInterval::new(start, end).occupancy(intervals.iter()),
            4 * 3 + 2 * 2,
        );
    }

    #[test]
    fn partial_interval_with_weights_responsibilities() {
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
        ];
        let group = RoommateGroup::new(vec![&Roommate::new("me"), &Roommate::new("someone")]);
        let split = group.individual_responsibilities(&intervals, DateInterval::new(start, end));
        let table: HashMap<_, _> = split.iter().collect();
        assert_eq!(
            **table.get(&intervals[0].roommate()).unwrap(),
            Ratio::new(2 * 2, 4 * 3 + 2 * 2),
        );
        assert_eq!(
            table.values().cloned().sum::<Ratio<u32>>(),
            Ratio::from_integer(1),
        );
    }

    #[test]
    fn no_overlap_between_billing_period_and_intervals() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![ResponsibilityInterval::new(
            Roommate::new("me"),
            1,
            DateInterval::new(
                NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
                NaiveDate::parse_from_str("02/02/19", "%D").unwrap(),
            ),
        )];
        let billing_period = DateInterval::new(start, end);
        let group = RoommateGroup::new(vec![&Roommate::new("me"), &Roommate::new("someone")]);
        assert_eq!(billing_period.occupancy(intervals.iter()), 0,);
        assert_eq!(
            **group
                .individual_responsibilities(&intervals, &billing_period)
                .iter()
                .collect::<HashMap<_, _>>()
                .get(&intervals[0].roommate())
                .unwrap(),
            Ratio::new(1, 2)
        );
    }
}
