use chrono::naive::NaiveDate;
use clap::{App, Arg};
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::File;

mod interval;
use interval::OwnedInterval;

fn main() {
    let matches = App::new("roommates")
        .version("0.1.0")
        .about("Calculating shared living expenses")
        .arg(
            Arg::with_name("start")
                .help("start of billing period")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("end")
                .help("end of billing period")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("intervals")
                .help("CSV file giving intervals when people were responsible")
                .required(true)
                .index(3),
        )
        .get_matches();

    let billing_period = NaiveDateInterval(
        date_from_arg_string(&matches, "start"),
        date_from_arg_string(&matches, "end"),
    );
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(
        File::open(matches.value_of("intervals").unwrap()).expect("Could not find intervals file"),
    );
    let intervals = rdr
        .records()
        .map(|r| OwnedInterval::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>();
    println!(
        "weighted value over billing period: {}",
        proportion_of_interval(&intervals, &billing_period)
    );
    let map = individual_responsibilities(&intervals, &billing_period);
    for name in map.keys() {
        println!("{}: {}", name, map.get(name).unwrap());
    }
}

#[derive(Debug)]
struct NaiveDateInterval(NaiveDate, NaiveDate);

fn date_from_arg_string(matches: &clap::ArgMatches, arg: &str) -> NaiveDate {
    NaiveDate::parse_from_str(matches.value_of(arg).unwrap(), "%D").expect(&format!(
        "Invalid {} time format. (Correct format: mm/dd/yy)",
        arg
    ))
}

fn proportion_of_interval(
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
        .fold(0, |acc, x| acc + x)
}

/// Returns the proportion of the total cost that each contributing party
/// is responsible for
fn individual_responsibilities(
    intervals: &Vec<OwnedInterval>,
    billing_period: &NaiveDateInterval,
) -> HashMap<String, f64> {
    intervals
        .iter()
        .map(|i| &i.owner_name)
        .unique()
        .map(|name| {
            (
                String::from(name),
                proportion_by_name(intervals, &billing_period, name),
            )
        })
        .collect()
}

fn proportion_by_name(
    intervals: &Vec<OwnedInterval>,
    billing_period: &NaiveDateInterval,
    name: &str,
) -> f64 {
    let val = total_cost_in_interval(
        &intervals.iter().filter(|i| &i.owner_name == name).collect(),
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
            owner_name: String::from("me"),
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
                owner_name: String::from("me"),
                weight: 2,
            },
            OwnedInterval {
                start,
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner_name: String::from("someone"),
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
                owner_name: String::from("me"),
                weight: 2,
            },
            OwnedInterval {
                start: NaiveDate::parse_from_str("01/10/19", "%D").unwrap(),
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner_name: String::from("someone"),
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
                owner_name: String::from("me"),
                weight: 2,
            },
            OwnedInterval {
                start,
                end: NaiveDate::parse_from_str("01/13/20", "%D").unwrap(),
                owner_name: String::from("someone"),
                weight: 4,
            },
        ];
        let table = individual_responsibilities(&intervals, &NaiveDateInterval(start, end));
        let total = (4.0 * 3.0 + 2.0 * 2.0) / 10.0;
        assert_eq!(*table.get("me").unwrap(), (2.0 * 2.0) / 10.0 as f64 / total);
        assert_eq!(table.values().sum::<f64>(), 1.0);
    }

    #[test]
    fn no_overlap_between_billing_period_and_intervals() {
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        let intervals = vec![OwnedInterval {
            start: NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
            end: NaiveDate::parse_from_str("01/02/19", "%D").unwrap(),
            owner_name: String::from("me"),
            weight: 1,
        }];
        let billing_period = NaiveDateInterval(start, end);
        assert_eq!(proportion_of_interval(&intervals, &billing_period), 0.0);
        assert_eq!(
            *individual_responsibilities(&intervals, &billing_period)
                .get("me")
                .unwrap(),
            0.0
        );
    }
}
