use std::fs::File;
use clap::{App, Arg};
use chrono::naive::NaiveDate;
use std::cmp::{max, min};

mod interval;
use interval::OwnedInterval;

fn main() {
    let matches = App::new("billing-period-proportion")
        .version("0.1.0")
        .about("Calculating stuff")
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
                .help("file giving intervals when people were responsible")
                .required(true)
                .index(3),
        )
        .get_matches();

    let start = date_from_arg_string(&matches, "start");
    let end = date_from_arg_string(&matches, "end");
    println!("start: {}, end: {}", start, end);
    println!(
        "Using input file: {}",
        matches.value_of("intervals").unwrap()
    );
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(File::open(matches.value_of("intervals").unwrap())
            .expect("Could not find intervals file")
        );
    let intervals = rdr.records()
        .map(|r| OwnedInterval::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>();
}

fn date_from_arg_string(matches: &clap::ArgMatches, arg: &str) -> NaiveDate {
    NaiveDate::parse_from_str(matches.value_of(arg).unwrap(), "%D").expect(&format!(
        "Invalid {} time format. (Correct format: mm/dd/yy)",
        arg
    ))
}

fn proportion_of_interval(intervals: Vec<OwnedInterval>, start: NaiveDate, end: NaiveDate) -> f64 {
    let total_weighted_days = intervals.iter()
        .map(|i| i.weight * min(i.end, end).signed_duration_since(max(i.start, start)).num_days() as u32)
        .fold(0, |acc, x| acc + x) as f64;
    total_weighted_days / end.signed_duration_since(start).num_days() as f64
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
            weight: 1
        }];
        assert_eq!(proportion_of_interval(intervals, start, end), 1.0);
    }

    #[test]
    fn partial_interval_with_weights() {
        let start = NaiveDate::parse_from_str("01/10/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("01/20/20", "%D").unwrap();
        let intervals = vec![OwnedInterval {
            start: NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
            end,
            owner_name: String::from("me"),
            weight: 2
        }, OwnedInterval {
                    start: NaiveDate::parse_from_str("01/18/20", "%D").unwrap(),
                    end,
                    owner_name: String::from("me"),
                    weight: 2
                }];
        assert_eq!(proportion_of_interval(intervals, start, end), 1.0);
    }
}
