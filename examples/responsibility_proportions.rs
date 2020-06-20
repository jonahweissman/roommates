use chrono::naive::NaiveDate;
use clap::{App, Arg, ArgMatches, SubCommand};
use csv::StringRecord;
use std::fs::File;

use roommates::responsibility::{self, interval::{ResponsibilityInterval, NaiveDateInterval}};
use roommates::roommate::Roommate;

fn main() {
    let matches = App::new("roommates")
        .version("0.1.0")
        .about("Calculating shared living expenses")
        .subcommand(
            SubCommand::with_name("split")
                .about("Determine individuals responsibilities in a billing period")
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
                ),
        )
        .subcommand(
            SubCommand::with_name("share")
                .about("Determine final dollar values to be paid by each roommmate")
                .arg(Arg::with_name("water").required(true).index(1)),
        )
        .get_matches();

    match matches.subcommand() {
        ("split", Some(sub_m)) => {
            split(sub_m);
        }
        _ => {
            println!("command not recognized");
        }
    }
}

fn date_from_arg_string(matches: &ArgMatches, arg: &str) -> NaiveDate {
    NaiveDate::parse_from_str(matches.value_of(arg).unwrap(), "%D").expect(&format!(
        "Invalid {} time format. (Correct format: mm/dd/yy)",
        arg
    ))
}

trait FromStringRecord {
    fn from_string_record(sr: StringRecord) -> Self;
}

impl FromStringRecord for ResponsibilityInterval {
    fn from_string_record(sr: StringRecord) -> Self {
        assert_eq!(sr.len(), 4, "Found row with wrong number of columns");
        let start = NaiveDate::parse_from_str(sr.get(2).expect("Missing start date"), "%m/%d/%Y")
            .expect("Invalid start date");
        let end = NaiveDate::parse_from_str(sr.get(3).expect("Missing end date"), "%m/%d/%Y")
            .expect("Invalid end date");
        let roommate = Roommate::new(String::from(sr.get(0).expect("Missing person")));
        let count = sr
            .get(1)
            .expect("Missing count")
            .parse::<u32>()
            .expect("Invalid count");
        ResponsibilityInterval::new(roommate, count, (start, end))
    }
}

fn split(matches: &ArgMatches) {
    let billing_period = NaiveDateInterval::new(
        date_from_arg_string(&matches, "start"),
        date_from_arg_string(&matches, "end"),
    );
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(
        File::open(matches.value_of("intervals").unwrap()).expect("Could not find intervals file"),
    );
    let intervals = rdr
        .records()
        .map(|r| ResponsibilityInterval::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>();
    println!(
        "weighted value over billing period: {}",
        responsibility::proportion_of_interval(&intervals, &billing_period)
    );
    let map = responsibility::individual_responsibilities(&intervals, &billing_period);
    for name in map.keys() {
        println!("{}: {}", name, map.get(name).unwrap());
    }
}

