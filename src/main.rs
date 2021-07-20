use serde::{Deserialize, Deserializer};
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
    #[serde(deserialize_with = "amount_to_cents")]
    cents: i32,
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
        println!(
            "    {}    ${}.{:02}",
            record.account,
            record.cents / 100,
            (record.cents % 100).abs(),
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

fn amount_to_cents<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    let amount: f64 = Deserialize::deserialize(deserializer)?;
    Ok((amount * 100.0) as i32)
}
