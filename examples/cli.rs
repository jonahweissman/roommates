use clap::{App, Arg, ArgGroup};
use csv::StringRecord;
use std::fs::File;
use steel_cent::formatting;

use roommates::bill::Bill;
use roommates::interval::{DateInterval, ResponsibilityInterval};
use roommates::roommate::{Roommate, RoommateGroup};

fn main() {
    let matches = App::new("roommates")
        .version("0.1.0")
        .about("Split a collection of bills based on when people were responsibile")
        .arg(
            Arg::with_name("intervals")
                .help("CSV file giving intervals when people were responsible")
                .takes_value(true)
                .value_name("INTERVALS.CSV")
                .required(true),
        )
        .arg(
            Arg::with_name("roommates")
                .help("names of all roommates sharing costs")
                .multiple(true)
                .takes_value(true)
                .required(true),
        )
        .group(
            ArgGroup::with_name("bills")
                .args(&["water bill", "electric bill"])
                .required(true),
        )
        .arg(
            Arg::with_name("water bill")
                .help("file listing water bill amounts and periods")
                .long("water")
                .takes_value(true)
                .value_name("WATER.CSV"),
        )
        .arg(
            Arg::with_name("electric bill")
                .help("file listing electric bill amounts and periods")
                .long("electric")
                .takes_value(true)
                .value_name("ELECTRIC.CSV"),
        )
        .arg(
            Arg::with_name("weather data")
                .help("NOAA data file to account for temperature variation")
                .long("weather")
                .requires("electric bill")
                .takes_value(true)
                .value_name("WEATHER.CSV"),
        )
        .get_matches();
    let intervals = build_intervals(matches.value_of("intervals").unwrap());
    let roommates = RoommateGroup::from_strs(matches.values_of("roommates").unwrap().collect());
    let mut water_bills = build_bills(matches.value_of("water bill").unwrap())
        .into_iter()
        .map(|b| (b, None))
        .collect::<Vec<_>>();
    let current_water = water_bills.remove(water_bills.len() - 3);
    let water_history = water_bills;
    let invoices = roommates.generate_invoices(vec![("water", current_water, water_history)], intervals);
    for invoice in invoices.iter() {
        println!("{}", invoice);
    }
}

trait FromStringRecord {
    fn from_string_record(sr: StringRecord) -> Self;
}

impl FromStringRecord for ResponsibilityInterval {
    fn from_string_record(sr: StringRecord) -> Self {
        assert_eq!(sr.len(), 4, "Found row with wrong number of columns");
        let period = DateInterval::from_strs(
            sr.get(2).expect("Missing start date"),
            sr.get(3).expect("Missing end date"),
        );
        let roommate = Roommate::new(sr.get(0).expect("Missing person"));
        let count = sr
            .get(1)
            .expect("Missing count")
            .parse::<u32>()
            .expect("Invalid count");
        ResponsibilityInterval::new(roommate, count, period)
    }
}

impl FromStringRecord for Bill {
    fn from_string_record(sr: StringRecord) -> Self {
        assert_eq!(sr.len(), 3, "Found row with wrong number of columns");
        let amount_due = formatting::us_style()
            .parser()
            .parse(sr.get(0).expect("no money"))
            .expect("Invalid money format");
        let period = DateInterval::from_strs(
            sr.get(1).expect("Missing start date"),
            sr.get(2).expect("Missing end date"),
        );
        Bill::new(amount_due, period, None)
    }
}

fn build_intervals(file_name: &str) -> Vec<ResponsibilityInterval> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(File::open(file_name).expect("Could not find intervals file"));
    rdr.records()
        .map(|r| ResponsibilityInterval::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>()
}

fn build_bills(file_name: &str) -> Vec<Bill> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(File::open(file_name).expect("Could not find bill file"));
    rdr.records()
        .map(|r| Bill::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>()
}
