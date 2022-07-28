use rand::Rng;
use colored::Colorize;
use clap::{AppSettings, Parser};
use std::{str::FromStr};
use ethers_core::{types::H160, utils::to_checksum};
use serde::{Serialize, Deserialize};
use serde_json::to_string_pretty;
use reqwest::{Response,Error};


/// A set of quick commands to speed up testing and development on Ethereum
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
// #[clap(global_setting(AppSettings::ArgRequiredElseHelp))]
struct CliArgs {
   /// Generate random checksummed addresses for testing 
   #[clap(short, long, value_parser)]
   generate: Option<u128>,

   /// Pass an ethereum address to validate the checksum
   #[clap(short, long, value_parser)]
   validate: Option<String>

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
        hex2 =  format!("{num2:x}");
    }
    // concatenate
    hex.push_str(&hex2); 
    return hex;
}

fn check_sum(val: &str) -> String {
    if val.len() != 42 {
        panic!("Running a checksum on an invalid ethereum address");
    } else {
        return to_checksum(
            &H160::from_str(val).expect(""), 
            None
        );
    }
}

// #[tokio::main]
// async fn main() {
//     // let args = CliArgs::parse();

//     // match args.generate {
//     //     Some(n) => { 
//     //         let message = match n {
//     //             0 => "".to_string(),
//     //             1 => "Generated 1 new Ethereum Address".to_string(),
//     //             2.. => format!("Generated {} new Ethereum Addresses", n)
//     //         };
//     //         println!("{}", message);
//     //         for _ in 0..n {
//     //             let raw_hex = generate_hex_address();
//     //             let ethereum_address = check_sum(&raw_hex);
//     //             println!("{}", format!("{}", ethereum_address).bold().blue());
//     //         }  
//     //     }
//     //     None => ()  
//     // }

//     // match args.validate {
//     //     Some(val) => {
//     //         let checksummed = check_sum(&val);
//     //         if val == checksummed {
//     //             println!("{}", format!("Checksum Valid").bold().green());
//     //         } else {
//     //             println!("{}", format!("Checksum Invalid - Valid Checksum Below:").bold().red());
//     //         }
//     //         println!("{}", format!("{}", checksummed).bold().blue())
//     //     },
//     //     None => ()
//     // };

//     // req().await;
//     let request_url = format!("https://api.github.com/repos/{owner}/{repo}/stargazers",
//     owner = "rust-lang-nursery",
//     repo = "rust-cookbook");
//     println!("{}", request_url);
//     let response = reqwest::get(&request_url).await?;

//     let users: Vec<User> = response.json().await;
//     // println!("{:?}", users);
    
// }

#[derive(Serialize, Deserialize, Debug)]
struct Holder {
    address: String,
    balance: u128,
    share: f64
}
#[derive(Serialize, Deserialize, Debug)]
struct Res {
    holders: Vec<Holder>,
}



async fn req() {
    let client = reqwest::Client::new();
    let request_url = format!("https://api.ethplorer.io/getTopTokenHolders/{token}?apiKey={key}&limit={limit}",
                                token = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                                key = "freekey",
                                limit = "5"
                            );
    let res = client
        .get(request_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "reqwest/0.11.11")
        .send()
        .await
        .unwrap();

    // let _res = &res.text().await.unwrap();
    let _res = res.json::<Res>().await.unwrap();
    
    println!("{}", to_string_pretty(&_res).unwrap());
}


#[tokio::main]
async fn main() {
    req().await;
}
