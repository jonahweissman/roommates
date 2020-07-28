use anyhow::{Context, Result};
use chrono::naive::NaiveDate;
use clap::{App, Arg, ArgGroup};
use csv::StringRecord;
use itertools::Itertools;
use num::rational::Ratio;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use steel_cent::{formatting, Money};

use roommates::sharing::{self, Bill, SharedBill};
use roommates::splitting::CostSplit;
use roommates::{DateInterval, ResponsibilityInterval, ResponsibilityRecord};
use roommates::{Roommate, RoommateGroup};

// try running
// > cargo run --example cli -- examples/responsibility_intervals.csv Rupert Georg Winifred Hestia Juan --electric examples/electric.csv --weather examples/weather.csv --water examples/water.csv --internet examples/internet.csv

fn main() -> Result<()> {
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
    let roommates: RoommateGroup = matches.values_of("roommates").unwrap().collect();
    let intervals = build_intervals(matches.value_of("intervals").unwrap(), &roommates)?;
    let mut bills = Vec::new();
    let current_bill_position_from_end = 1;
    if let Some(file_name) = matches.value_of("water bill") {
        let mut water_bills = build_bills(file_name)?;
        let current_water = water_bills.remove(water_bills.len() - current_bill_position_from_end);
        let water = sharing::convert_to_shared((current_water, water_bills), &intervals)?;
        bills.push(("water", water));
    }
    if let Some(file_name) = matches.value_of("internet bill") {
        let mut internet_bills = build_bills(file_name)?;
        let current_internet = SharedBill::from_fully_fixed(
            internet_bills.remove(internet_bills.len() - current_bill_position_from_end),
        );
        bills.push(("internet", current_internet));
    }
    if let Some(file_name) = matches.value_of("electric bill") {
        let weather_data = WeatherData::build(matches.value_of("weather data").unwrap())?;
        let mut electric_bills = build_bills(file_name)?
            .into_iter()
            .map(|bill| {
                let ti = weather_data.calculate_temperature_index(&bill);
                (bill, ti)
            })
            .collect::<Vec<(Bill, f64)>>();
        let current_electric =
            electric_bills.remove(electric_bills.len() - current_bill_position_from_end);
        let electric =
            sharing::convert_to_shared_ti((current_electric, electric_bills), &intervals)?;
        bills.push(("electric", electric));
    }
    let invoices = generate_invoices(&roommates, bills, &intervals);
    for invoice in invoices.iter() {
        println!("{}", invoice);
    }
    Ok(())
}

trait FromStringRecord: Sized {
    fn from_string_record(sr: StringRecord) -> Result<Self>;
}

impl FromStringRecord for Bill {
    fn from_string_record(sr: StringRecord) -> Result<Self> {
        assert_eq!(sr.len(), 3, "Found row with wrong number of columns");
        let amount_due = formatting::us_style()
            .parser()
            .parse(sr.get(0).context("no money")?)
            .context("Invalid money format")?;
        let period = DateInterval::from_strs(
            sr.get(1).context("Missing start date")?,
            sr.get(2).context("Missing end date")?,
        )
        .context("invalid interval")?;
        Ok(Bill::new(amount_due, period))
    }
}

fn build_intervals<'a, 'b>(
    file_name: &'a str,
    roommate_group: &'b RoommateGroup,
) -> Result<ResponsibilityRecord<'b>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(File::open(file_name).context("Could not find intervals file")?);
    let mut intervals = vec![];
    for sr in rdr.records() {
        let sr = sr.context("bad record")?;
        assert_eq!(sr.len(), 4, "Found row with wrong number of columns");
        let period = DateInterval::from_strs(
            sr.get(2).context("Missing start date")?,
            sr.get(3).context("Missing end date")?,
        )
        .context("invalid interval")?;
        let roommate_name = sr.get(0).context("Missing person")?;
        let roommate = if let Some(roommate) = roommate_group.borrow_by_name(roommate_name) {
            roommate
        } else {
            println!(
                "skipping responsibility interval with unknown roommate: {}",
                roommate_name
            );
            continue;
        };
        let guests = sr
            .get(1)
            .context("Missing guest count")?
            .parse::<u32>()
            .context("Invalid guess count")?;
        intervals.push(ResponsibilityInterval::new(roommate, period, guests));
    }
    Ok(intervals.into_iter().collect())
}

fn build_bills(file_name: &str) -> Result<Vec<Bill>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(File::open(file_name).context("Could not find bill file")?);
    Ok(rdr
        .records()
        .map(|r| Ok(Bill::from_string_record(r.context("bad record")?)?))
        .collect::<Result<Vec<_>>>()?)
}

struct WeatherData {
    data: Vec<(NaiveDate, f64, f64)>,
}
impl WeatherData {
    fn build(file_name: &str) -> Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .from_reader(File::open(file_name).context("Could not find weather file")?);
        let data = rdr
            .records()
            .map(|r| {
                let r = r.context("bad record")?;
                Ok((
                    NaiveDate::parse_from_str(r.get(2).unwrap(), "%Y-%m-%d").context("bad date")?,
                    r.get(3).unwrap().parse::<f64>().unwrap(),
                    r.get(4).unwrap().parse::<f64>().unwrap(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(WeatherData { data })
    }

    fn calculate_temperature_index(&self, bill: &Bill) -> f64 {
        let (start, end) = (bill.usage_period().start(), bill.usage_period().end());
        self.data
            .iter()
            .filter(|(x, _, _)| x > &start && x < &end)
            .fold(0.0, |a, (_, low, high)| {
                a + temperature_index((*low, *high))
            })
    }
}

fn temperature_index((low, high): (f64, f64)) -> f64 {
    ((low + high) / 2.0 - 70.0).powf(2.0)
}

pub struct Invoice {
    to: Roommate,
    total: Money,
    components: Vec<InvoiceComponent>,
}

struct InvoiceComponent {
    label: String,
    amount_due: Money,
    shared_amount: Money,
    responsibility_proportion: Ratio<u32>,
}

pub fn generate_invoices<'a, I>(
    roommate_group: &RoommateGroup,
    shared_bills: I,
    responsibility_intervals: &ResponsibilityRecord,
) -> Vec<Invoice>
where
    I: IntoIterator<Item = (&'a str, SharedBill)>,
{
    let mut invoice_components: HashMap<Roommate, Vec<InvoiceComponent>> = HashMap::new();
    let bill_list = shared_bills
        .into_iter()
        .map(|(label, shared_bill)| {
            let split = roommate_group
                .individual_responsibilities(responsibility_intervals, shared_bill.usage_period());
            for (roommate, share) in split.hash_map() {
                invoice_components
                    .entry(roommate.clone())
                    .or_insert_with(|| vec![])
                    .push(InvoiceComponent {
                        label: String::from(label),
                        responsibility_proportion: share,
                        amount_due: shared_bill.amount_due(),
                        shared_amount: shared_bill.shared_amount(),
                    })
            }
            (shared_bill, split)
        })
        .collect::<Vec<_>>();
    CostSplit::split_bill_list(bill_list.iter().map(|(b, s)| (b, s)))
        .into_iter()
        .map(|(to, total)| {
            let components = invoice_components.remove(to).unwrap();
            let to = to.clone();
            Invoice {
                to,
                total,
                components,
            }
        })
        .collect()
}

impl fmt::Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} owes {}\n{}",
            self.to,
            self.total,
            self.components.iter().join("\n")
        )
    }
}

impl fmt::Display for InvoiceComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\t{} of the responsibility for the {} non-shared portion of the {} {} bill",
            self.responsibility_proportion,
            self.amount_due - self.shared_amount,
            self.amount_due,
            self.label
        )
    }
}
