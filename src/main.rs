use clap::{AppSettings, Parser};
use colored::Colorize;
use dotenv;
use ethers::{types::H160, utils::to_checksum};
use rand::Rng;
use std::str::FromStr;

mod whale;

/// A set of quick commands to speed up testing and development on Ethereum
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::ArgRequiredElseHelp))]
struct CliArgs {
    /// Generate random checksummed addresses for testing
    #[clap(short, long, value_parser)]
    generate: Option<u128>,

    /// Pass an ethereum address to validate the checksum
    #[clap(short, long, value_parser)]
    validate: Option<String>,
}

fn generate_hex_address() -> String {
    let mut rng = rand::thread_rng();

    let mut hex: String = "".to_string();
    let mut hex2: String = "".to_string();

    // generates first 32 characters prefixed with 0x
    while hex.len() < 34 {
        let num = rng.gen::<u128>();
        hex = format!("{num:#x}");
    }

    // generate remainder 8 character
    while hex2.len() < 8 {
        let num2 = rng.gen::<u32>();
        hex2 = format!("{num2:x}");
    }
    // concatenate
    hex.push_str(&hex2);
    return hex;
}

fn check_sum(val: &str) -> String {
    if val.len() != 42 {
        panic!("Running a checksum on an invalid ethereum address");
    } else {
        return to_checksum(&H160::from_str(val).expect(""), None);
    }
}

pub fn cli() {
    let args = CliArgs::parse();

    match args.generate {
        Some(n) => {
            let message = match n {
                0 => "".to_string(),
                1 => "Generated 1 new Ethereum Address".to_string(),
                2.. => format!("Generated {} new Ethereum Addresses", n),
            };
            println!("{}", message);
            for _ in 0..n {
                let raw_hex = generate_hex_address();
                let ethereum_address = check_sum(&raw_hex);
                println!("{}", format!("{}", ethereum_address).bold().blue());
            }
        }
        None => (),
    }

    match args.validate {
        Some(val) => {
            let checksummed = check_sum(&val);
            if val == checksummed {
                println!("{}", format!("Checksum Valid").bold().green());
            } else {
                println!(
                    "{}",
                    format!("Checksum Invalid - Valid Checksum Below:")
                        .bold()
                        .red()
                );
            }
            println!("{}", format!("{}", checksummed).bold().blue())
        }
        None => (),
    };
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    //
    let token: String = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string();

    whale::get_whale_balances_for(token).await;
}
