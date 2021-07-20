use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::process;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    date: Option<String>,
    description: Option<String>,
    #[serde(rename = "Full Account Name")]
    account: String,
    #[serde(rename = "Amount Num")]
    amount: String,
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let mut rdr = csv::Reader::from_path(file_path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        if let (Some(date), Some(description)) = (record.date, record.description) {
            println!("\n{} {}", date, description);
        }
        let amount = amount_to_cents(&record.amount).unwrap();
        println!(
            "    {}    ${}.{}",
            record.account,
            amount / 100,
            (amount % 100).abs()
        );
    }
    Ok(())
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn amount_to_cents(amount: &str) -> Option<i32> {
    lazy_static! {
        // Matches signed integers and decimals of up to 2 places
        static ref RE: Regex = Regex::new(r"^(-)?(\d+)(?:.(\d{1,2}))?$").unwrap();
    }
    let caps = RE.captures(amount)?;
    let sign = if caps.get(1).is_none() { 1 } else { -1 };
    // If match exists then parse should always succeed
    let integer = caps.get(2)?.as_str().parse::<i32>().unwrap();
    let decimal = caps
        .get(3)
        // If match exists then parse should always succeed
        .map(|x| x.as_str().parse::<i32>().unwrap())
        .unwrap_or(0);
    Some(sign * (100 * integer + decimal))
}
