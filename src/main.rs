use serde::Deserialize;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::io::Read;
use std::process;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    date: Option<String>,
    description: Option<String>,
    #[serde(rename = "Notes")]
    comment: Option<String>,
    #[serde(rename = "Full Account Name")]
    account: String,
    #[serde(rename = "Amount Num")]
    amount: f64,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let reader = csv::Reader::from_path(file_path)?;
    let ledger = convert(reader)?;
    print!("{}", ledger);
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

fn convert<R: Read>(mut reader: csv::Reader<R>) -> Result<String, Box<dyn Error>> {
    let mut ledger = String::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        if let (Some(date), Some(description)) = (record.date, record.description) {
            write!(&mut ledger, "\n{} {}\n", date, description)?;
        }
        if let Some(comment) = record.comment {
            for line in comment.split('\n') {
                write!(&mut ledger, "    ; {}\n", line)?;
            }
        }
        write!(
            &mut ledger,
            "    {}    ${:.2}\n",
            record.account, record.amount
        )?;
    }
    Ok(ledger)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod convert {
        use super::*;
        use insta;

        #[test]
        fn transaction() {
            let data = "\
Date,Transaction ID,Number,Description,Notes,Commodity/Currency,Void Reason,Action,Memo,Full Account Name,Account Name,Amount With Sym.,Amount Num,Reconcile,Reconcile Date,Rate/Price
2000-01-01,00000000000000000000000000000000,,Shop,,CURRENCY::AUD,,,,Expenses:Groceries,Groceries,$10.00,10.00,n,,1.00
,,,,,,,,,Assets:Wallet,Wallet,-$10.00,-10.00,n,,1.00
";
            let reader = csv::Reader::from_reader(data.as_bytes());
            let result = convert(reader).unwrap();
            insta::assert_snapshot!(result, @r###"

            2000-01-01 Shop
                Expenses:Groceries    $10.00
                Assets:Wallet    $-10.00
            "###);
        }

        #[test]
        fn transaction_with_three_postings() {
            let data = "\
Date,Transaction ID,Number,Description,Notes,Commodity/Currency,Void Reason,Action,Memo,Full Account Name,Account Name,Amount With Sym.,Amount Num,Reconcile,Reconcile Date,Rate/Price
2000-01-01,00000000000000000000000000000000,,Shop,,CURRENCY::AUD,,,,Expenses:Groceries,Groceries,$20.00,20.00,n,,1.00
,,,,,,,,,Assets:Wallet,Wallet,-$10.00,-10.00,n,,1.00
,,,,,,,,,Assets:Bank,Bank,-$10.00,-10.00,n,,1.00
";
            let reader = csv::Reader::from_reader(data.as_bytes());
            let result = convert(reader).unwrap();
            insta::assert_snapshot!(result, @r###"

            2000-01-01 Shop
                Expenses:Groceries    $20.00
                Assets:Wallet    $-10.00
                Assets:Bank    $-10.00
            "###);
        }

        #[test]
        fn transaction_with_comment() {
            let data = "\
Date,Transaction ID,Number,Description,Notes,Commodity/Currency,Void Reason,Action,Memo,Full Account Name,Account Name,Amount With Sym.,Amount Num,Reconcile,Reconcile Date,Rate/Price
2000-01-01,00000000000000000000000000000000,,Shop,Comment,CURRENCY::AUD,,,,Expenses:Groceries,Groceries,$10.00,10.00,n,,1.00
,,,,,,,,,Assets:Wallet,Wallet,-$10.00,-10.00,n,,1.00
";
            let reader = csv::Reader::from_reader(data.as_bytes());
            let result = convert(reader).unwrap();
            insta::assert_snapshot!(result, @r###"

            2000-01-01 Shop
                ; Comment
                Expenses:Groceries    $10.00
                Assets:Wallet    $-10.00
            "###);
        }

        #[test]
        fn transaction_with_multi_line_comment() {
            let data = "\
Date,Transaction ID,Number,Description,Notes,Commodity/Currency,Void Reason,Action,Memo,Full Account Name,Account Name,Amount With Sym.,Amount Num,Reconcile,Reconcile Date,Rate/Price
2000-01-01,00000000000000000000000000000000,,Shop,\"Comment 1\nComment 2\",CURRENCY::AUD,,,,Expenses:Groceries,Groceries,$10.00,10.00,n,,1.00
,,,,,,,,,Assets:Wallet,Wallet,-$10.00,-10.00,n,,1.00
";
            let reader = csv::Reader::from_reader(data.as_bytes());
            let result = convert(reader).unwrap();
            insta::assert_snapshot!(result, @r###"

            2000-01-01 Shop
                ; Comment 1
                ; Comment 2
                Expenses:Groceries    $10.00
                Assets:Wallet    $-10.00
            "###);
        }
    }
}
