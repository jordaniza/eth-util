use clap::{AppSettings, Args, Parser, Subcommand};
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
    #[clap(short, long, value_parser, value_name = "QTY")]
    generate: Option<u128>,

    /// Pass an ethereum address to validate the checksum
    #[clap(short, long, value_parser, value_name = "ADDRESS")]
    validate: Option<String>,

    /// Pass an ethereum address to validate the checksum
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch a list of whales 🐋 for a given token
    Whales(Whale),
}

#[derive(Args, Debug)]
struct Whale {
    /// the ethereum token address to fetch whales for
    #[clap(short, value_parser)]
    token: String,

    /// How many whales to generate
    #[clap(short, value_parser, default_value_t = 5)]
    number: u8,

    #[clap(short, value_parser, default_value = "balances.json")]
    output_file: String,
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

async fn cli() {
    let args = CliArgs::parse();

    if let Some(generate) = args.generate {
        let message = match generate {
            0 => unreachable!(),
            1 => "Generated 1 new Ethereum Address".to_string(),
            2.. => format!("Generated {} new Ethereum Addresses", generate),
        };
        println!("{}", message);
        //
        for _ in 0..generate {
            let raw_hex = generate_hex_address();
            let ethereum_address = check_sum(&raw_hex);
            println!("{}", format!("{}", ethereum_address).bold().blue());
        }
    }

    if let Some(val) = args.validate {
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
    };

    if let Some(command) = args.command {
        match command {
            Commands::Whales(whale) => {
                let balances = whale::get_whale_balances_for(whale.token, whale.number).await;
                whale::write_to_file(&balances, whale.output_file);
            }
        };
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    cli().await;
}
