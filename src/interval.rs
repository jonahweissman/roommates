use chrono::format::ParseError;
use chrono::{naive::NaiveDate, Duration};
use std::cmp::{max, min};
use std::iter::FromIterator;
use std::slice::Iter;
use thiserror::Error;

use super::roommate::Roommate;

#[derive(Debug, Error)]
pub enum IntervalError {
    #[error("The end of an interval cannot be before the start")]
    NegativeLengthInterval,

    #[error("Error parsing date")]
    InvalidDate {
        #[from]
        source: ParseError,
    },
}

/// A continuous interval that someone stayed in the house
///
/// This will always includes one person, but can also include additional people.
/// Exactly one [`Roommate`] must have responsibility for any group or individual
/// that stays in the housing unit.
///
/// `ResponsibilityInterval`s can be collected in a [`ResponsibilityRecord`].
///
/// [`Roommate`]: struct.Roommate.html
/// [`ResponsibilityRecord`]: struct.ResponsibilityRecord.html
#[derive(Clone)]
pub struct ResponsibilityInterval<'a> {
    roommate: &'a Roommate,
    interval: DateInterval,
    additional_people: u32,
}

impl<'a> ResponsibilityInterval<'a> {
    /// Creates a new `ResponsibilityInterval`
    ///
    /// Note that the third argument is `additional_people`; the
    /// total number of people represented by the `ResponsibilityInterval`
    /// will be one more than the number given.
    ///
    /// # Examples
    /// ```
    /// use roommates::{Roommate, DateInterval, ResponsibilityInterval};
    ///
    /// let joe = Roommate::new("Joe");
    /// let joes_friends_visiting = ResponsibilityInterval::new(
    ///     &joe,
    ///     DateInterval::from_strs("01/15/2020", "01/22/2020").unwrap(),
    ///     1,
    /// );
    /// ```
    pub fn new(roommate: &'a Roommate, interval: DateInterval, additional_people: u32) -> Self {
        ResponsibilityInterval {
            roommate,
            interval,
            additional_people,
        }
    }

    /// Returns a reference to the `Roommate` financially responsible for the interval
    ///
    /// # Examples
    /// ```
    /// use roommates::{Roommate, DateInterval, ResponsibilityInterval};
    ///
    /// let joe = Roommate::new("Joe");
    /// let joes_friends_visiting = ResponsibilityInterval::new(
    ///     &joe,
    ///     DateInterval::from_strs("01/15/2020", "01/22/2020").unwrap(),
    ///     1,
    /// );
    /// assert_eq!(joes_friends_visiting.roommate(), &joe);
    /// ```
    pub fn roommate(&self) -> &Roommate {
        self.roommate
    }

    /// Returns the total number of people represented by the `ResponsibilityInterval`
    ///
    /// # Examples
    /// ```
    /// use roommates::{Roommate, DateInterval, ResponsibilityInterval};
    ///
    /// let joe = Roommate::new("Joe");
    /// let joes_friends_visiting = ResponsibilityInterval::new(
    ///     &joe,
    ///     DateInterval::from_strs("01/15/2020", "01/22/2020").unwrap(),
    ///     1,
    /// );
    /// assert_eq!(joes_friends_visiting.num_people(), 2);
    /// ```
    pub fn num_people(&self) -> u32 {
        // the 1 represents the person implicit in the ResponsibilityInterval
        // (because all ResponsibilityIntervals represent at least one person)
        1 + self.additional_people
    }
}

/// A complete occupancy history described with [`ResponsibilityInterval`]s
///
/// Contains a set of `ResponsibilityInterval`s that form the definitive record
/// of who was in the housing unit, when they were there, and who is responsible
/// for the expenses they incur.
///
/// [`ResponsibilityInterval`]: struct.ResponsibilityInterval.html
pub struct ResponsibilityRecord<'a> {
    intervals: Vec<ResponsibilityInterval<'a>>,
}

impl ResponsibilityRecord<'_> {
    /// Returns an `Iterator` that visits each [`ResponsibilityInterval`] in the
    /// `ResponsibilityRecord`
    ///
    /// [`ResponsibilityInterval`]: struct.ResponsibilityInterval.html
    /// # Examples
    /// ```
    /// use roommates::{Roommate, ResponsibilityInterval, DateInterval, ResponsibilityRecord};
    ///
    /// let bob = Roommate::new("Bob");
    /// let joe = Roommate::new("Joe");
    /// let records: ResponsibilityRecord = vec![
    ///     ResponsibilityInterval::new(
    ///         &bob,
    ///         DateInterval::from_strs("01/15/2020", "01/20/2020").unwrap(),
    ///         0,
    ///     ),
    ///     ResponsibilityInterval::new(
    ///         &joe,
    ///         DateInterval::from_strs("01/10/2020", "01/15/2020").unwrap(),
    ///         0,
    ///     ),
    /// ].into_iter().collect();
    /// let mut record_iter = records.iter();
    /// assert_eq!(record_iter.next().unwrap().roommate(), &bob);
    /// assert_eq!(record_iter.next().unwrap().roommate(), &joe);
    /// ```
    pub fn iter(&self) -> Iter<ResponsibilityInterval> {
        self.intervals.iter()
    }

    /// Occupancy over a given interval
    ///
    /// The occupancy represented by the responsibility intervals is
    /// measured in `person * day` units.
    ///
    /// # Examples
    /// ```
    /// use roommates::{Roommate, ResponsibilityInterval, DateInterval, ResponsibilityRecord};
    ///
    /// let bob = Roommate::new("Bob");
    /// let joe = Roommate::new("Joe");
    /// let records: ResponsibilityRecord = vec![
    ///     ResponsibilityInterval::new(
    ///         &bob,
    ///         DateInterval::from_strs("01/10/2020", "01/19/2020").unwrap(),
    ///         0,
    ///     ),
    ///     ResponsibilityInterval::new(
    ///         &joe,
    ///         DateInterval::from_strs("01/10/2020", "01/14/2020").unwrap(),
    ///         0,
    ///     ),
    /// ].into_iter().collect();
    /// assert_eq!(
    ///     records.occupancy_over(DateInterval::from_strs("01/10/2020", "01/16/2020").unwrap()),
    ///     12,
    /// );
    /// assert_eq!(
    ///     records.occupancy_over(DateInterval::from_strs("01/12/2020", "01/12/2020").unwrap()),
    ///     2,
    /// );
    /// assert_eq!(
    ///     records.occupancy_over(DateInterval::from_strs("01/01/2020", "01/01/2021").unwrap()),
    ///     15,
    /// );
    /// assert_eq!(
    ///     records.occupancy_over(DateInterval::from_strs("05/10/2020", "05/16/2020").unwrap()),
    ///     0,
    /// );
    /// ```
    pub fn occupancy_over(&self, period: DateInterval) -> u32 {
        self.iter()
            .map(|r| r.num_people() * r.interval.num_days_bounded_by(period))
            .sum()
    }
}

impl<'a> FromIterator<ResponsibilityInterval<'a>> for ResponsibilityRecord<'a> {
    fn from_iter<I: IntoIterator<Item = ResponsibilityInterval<'a>>>(iter: I) -> Self {
        let intervals = iter.into_iter().collect::<Vec<_>>();
        ResponsibilityRecord { intervals }
    }
}

/// The time between a start date and an end date, inclusive
///
/// Does not store timezone information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateInterval(NaiveDate, NaiveDate);

impl DateInterval {
    /// Creates a new `DateInterval` from `(year, month, day)` tuples
    ///
    /// Returns a [`NegativeLengthInterval`] error if the end date is before
    /// the start date.
    ///
    /// [`NegativeLengthInterval`]: enum.IntervalError.html#variant.NegativeLengthInterval
    ///
    /// # Examples
    /// ```
    /// use roommates::DateInterval;
    ///
    /// assert!(DateInterval::new((2020, 1, 1), (2020, 3, 1)).is_ok());
    /// assert!(DateInterval::new((2020, 3, 1), (2020, 1, 1)).is_err());
    /// ```
    ///
    /// # Panics
    /// Panics on invalid dates.
    ///
    /// ```should_panic
    /// # use roommates::DateInterval;
    /// DateInterval::new((2020, 1, 32), (2020, 3, 1));
    /// ```
    pub fn new(
        (start_year, start_month, start_day): (i32, u32, u32),
        (end_year, end_month, end_day): (i32, u32, u32),
    ) -> Result<Self, IntervalError> {
        let start = NaiveDate::from_ymd(start_year, start_month, start_day);
        let end = NaiveDate::from_ymd(end_year, end_month, end_day);
        Ok(DateInterval::create_interval(start, end)?)
    }

    /// Creates a new `DateInterval` from `"month/day/year"` strings
    ///
    /// Returns a [`NegativeLengthInterval`] error if the end date is before
    /// the start date. Returns an [`InvalidDate`] error if either string
    /// cannot be parsed as a date in the expected format.
    ///
    /// [`NegativeLengthInterval`]: enum.IntervalError.html#variant.NegativeLengthInterval
    /// [`InvalidDate`]: enum.IntervalError.html#variant.InvalidDate
    ///
    /// # Examples
    /// ```
    /// use roommates::DateInterval;
    ///
    /// assert!(DateInterval::from_strs("01/01/2020", "12/01/2020").is_ok());
    /// assert!(DateInterval::from_strs("12/01/2020", "01/01/2020").is_err());
    /// assert!(DateInterval::from_strs("01/01/2020", "13/01/2020").is_err());
    /// assert!(DateInterval::from_strs("01-01-2020", "12-01-2020").is_err());
    pub fn from_strs(start: &str, end: &str) -> Result<Self, IntervalError> {
        let start = NaiveDate::parse_from_str(start, "%m/%d/%Y")
            .map_err(|source| IntervalError::InvalidDate { source })?;
        let end = NaiveDate::parse_from_str(end, "%m/%d/%Y")
            .map_err(|source| IntervalError::InvalidDate { source })?;
        Ok(DateInterval::create_interval(start, end)?)
    }

    fn create_interval(start: NaiveDate, end: NaiveDate) -> Result<Self, IntervalError> {
        if start > end {
            return Err(IntervalError::NegativeLengthInterval);
        }
        Ok(DateInterval(start, end))
    }

    /// The first day of the interval
    ///
    /// # Examples
    /// ```
    /// use roommates::DateInterval;
    /// use chrono::naive::NaiveDate;
    ///
    /// let january = DateInterval::new((2020, 1, 1), (2020, 1, 31)).unwrap();
    /// assert_eq!(january.start(), NaiveDate::from_ymd(2020, 1, 1));
    /// assert_eq!(january.end(), NaiveDate::from_ymd(2020, 1, 31));
    /// ```
    pub fn start(self) -> NaiveDate {
        self.0
    }

    /// The last day of the interval
    ///
    /// # Examples
    /// ```
    /// use roommates::DateInterval;
    /// use chrono::naive::NaiveDate;
    ///
    /// let january = DateInterval::new((2020, 1, 1), (2020, 1, 31)).unwrap();
    /// assert_eq!(january.start(), NaiveDate::from_ymd(2020, 1, 1));
    /// assert_eq!(january.end(), NaiveDate::from_ymd(2020, 1, 31));
    /// ```
    pub fn end(self) -> NaiveDate {
        self.1
    }

    /// The number of days in the interval that lie in a second interval
    ///
    /// This has the effect of truncating the interval and then counting
    /// the days in it.
    fn num_days_bounded_by(self, bounds: DateInterval) -> u32 {
        // to make it inclusive, we must move the end of both intervals
        // back by one day
        let signed_duration = min(
            self.end() + Duration::days(1),
            bounds.end() + Duration::days(1),
        )
        .signed_duration_since(max(self.start(), bounds.start()))
        .num_days();
        max(0, signed_duration) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_interval() {
        let start = (2020, 1, 2);
        let end = (2020, 2, 2);
        let me = Roommate::new("me");
        let intervals = vec![ResponsibilityInterval::new(
            &me,
            DateInterval::new(start, end).unwrap(),
            0,
        )]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(
            intervals.occupancy_over(DateInterval::new(start, end).unwrap()),
            32,
        );
    }

    #[test]
    fn partial_interval_with_weights() {
        let start = (2020, 1, 10);
        let end = (2020, 1, 20);
        let me = Roommate::new("me");
        let someone = Roommate::new("someone");
        let intervals = vec![
            ResponsibilityInterval::new(&me, DateInterval::new((2020, 1, 18), end).unwrap(), 1),
            ResponsibilityInterval::new(
                &someone,
                DateInterval::new(start, (2020, 1, 13)).unwrap(),
                3,
            ),
        ]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(
            intervals.occupancy_over(DateInterval::new(start, end).unwrap()),
            4 * 4 + 2 * 3,
        );
    }

    #[test]
    fn capping_intervals() {
        let start = (2020, 1, 10);
        let end = (2020, 1, 20);
        let me = Roommate::new("me");
        let someone = Roommate::new("someone");
        let intervals = vec![
            ResponsibilityInterval::new(
                &me,
                DateInterval::new((2020, 1, 18), (2021, 1, 21)).unwrap(),
                1,
            ),
            ResponsibilityInterval::new(
                &someone,
                DateInterval::new((2019, 1, 10), (2020, 1, 13)).unwrap(),
                3,
            ),
        ]
        .into_iter()
        .collect::<ResponsibilityRecord>();
        assert_eq!(
            intervals.occupancy_over(DateInterval::new(start, end).unwrap()),
            4 * 4 + 2 * 3,
        );
    }

    #[test]
    fn intervals_bounding() {
        let april = DateInterval::new((2020, 4, 1), (2020, 4, 30)).unwrap();
        let spring = DateInterval::new((2020, 3, 20), (2020, 6, 19)).unwrap();
        assert_eq!(april.num_days_bounded_by(spring), 30);
        assert_eq!(spring.num_days_bounded_by(april), 30);
    }
}
