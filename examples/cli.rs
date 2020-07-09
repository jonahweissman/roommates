use chrono::naive::NaiveDate;
use clap::{App, Arg, ArgGroup};
use csv::StringRecord;
use std::fs::File;
use steel_cent::formatting;

use roommates::bill::Bill;
use roommates::interval::{DateInterval, ResponsibilityInterval};
use roommates::invoice::SharingData::{Fixed, Variable};
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
                .args(&["water bill", "electric bill", "internet bill"])
                .required(true)
                .multiple(true),
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
                .requires("weather data")
                .value_name("ELECTRIC.CSV"),
        )
        .arg(
            Arg::with_name("internet bill")
                .help("file listing internet bill amounts and periods")
                .long("internet")
                .takes_value(true)
                .value_name("INTERNET.CSV"),
        )
        .arg(
            Arg::with_name("weather data")
                .help("NOAA data file to account for temperature variation")
                .long("weather")
                .takes_value(true)
                .value_name("WEATHER.CSV"),
        )
        .get_matches();
    let intervals = build_intervals(matches.value_of("intervals").unwrap());
    let roommates = RoommateGroup::from_strs(matches.values_of("roommates").unwrap().collect());
    let mut bills = Vec::new();
    let current_bill_position_from_end = 1;
    if let Some(file_name) = matches.value_of("water bill") {
        let mut water_bills = build_bills(file_name)
            .into_iter()
            .map(|b| (b, None))
            .collect::<Vec<_>>();
        let current_water = water_bills.remove(water_bills.len() - current_bill_position_from_end);
        bills.push(("water", Variable(current_water, water_bills)));
    }
    if let Some(file_name) = matches.value_of("internet bill") {
        let mut internet_bills = build_bills(file_name);
        let current_internet =
            internet_bills.remove(internet_bills.len() - current_bill_position_from_end);
        bills.push(("internet", Fixed(current_internet)));
    }
    if let Some(file_name) = matches.value_of("electric bill") {
        let weather_data = WeatherData::build(matches.value_of("weather data").unwrap());
        let mut electric_bills = build_bills(file_name)
            .into_iter()
            .map(|bill| {
                let ti = Some(weather_data.calculate_temperature_index(&bill));
                (bill, ti)
                })
            .collect::<Vec<_>>();
        let current_electric =
            electric_bills.remove(electric_bills.len() - current_bill_position_from_end);
        bills.push(("electric", Variable(current_electric, electric_bills)));
    }
    let invoices = roommates.generate_invoices(bills, intervals);
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

struct WeatherData {
    data: Vec<(NaiveDate, f64, f64)>,
}
impl WeatherData {
    fn build(file_name: &str) -> Self {
        let mut rdr = csv::ReaderBuilder::new()
            .from_reader(File::open(file_name).expect("Could not find weather file"));
        let data = rdr
            .records()
            .map(|r| r.expect("bad record"))
            .map(|r| {
                (
                    NaiveDate::parse_from_str(r.get(2).unwrap(), "%Y-%m-%d").expect("bad date"),
                    r.get(4).unwrap().parse::<f64>().unwrap(),
                    r.get(5).unwrap().parse::<f64>().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        WeatherData { data }
    }

    fn calculate_temperature_index(&self, bill: &Bill) -> f64 {
        let (start, end) = bill.period().interval();
        self.data
            .iter()
            .filter(|(x, _, _)| x > &start && x < &end)
            .fold(0.0, |a, (_, low, high)| a + temperature_index((*low, *high)))
    }
}

fn temperature_index((low, high): (f64, f64)) -> f64 {
    ((low + high) / 2.0 - 70.0).powf(2.0)
}
