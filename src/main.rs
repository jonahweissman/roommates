use chrono::naive::NaiveDate;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;

mod split;
use split::interval::OwnedInterval;
use split::NaiveDateInterval;

mod share;

mod roommate;

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
        .map(|r| OwnedInterval::from_string_record(r.expect("bad record")))
        .collect::<Vec<_>>();
    println!(
        "weighted value over billing period: {}",
        split::proportion_of_interval(&intervals, &billing_period)
    );
    let map = split::individual_responsibilities(&intervals, &billing_period);
    for name in map.keys() {
        println!("{}: {}", name, map.get(name).unwrap());
    }
}

fn share(matches: &ArgMatches) {}
