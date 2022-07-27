use rand::Rng;
use colored::Colorize;
use clap::Parser;

/// A set of quick commands to speed up testing and development on Ethereum
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
   /// Generate random addresses for testing 
   #[clap(short, long, value_parser, default_value_t = 1)]
   address: u128

}


fn generate_hex_address() -> String {
    let mut rng = rand::thread_rng();
    let num = rng.gen::<u128>();
    let num2 = rng.gen::<u32>();

    // generates first characters prefixed with 0x
    let mut hex = format!("{num:#x}");

    // generate remainder
    let hex2 = format!("{num2:x}");
    
    // concatenate
    hex.push_str(&hex2); 
    return hex;
}

fn main() {
    let args = CliArgs::parse();
    let n = args.address;

    println!("Generated {} new Ethereum Address(es)", n);
    for _ in 0..n {
        let hex = generate_hex_address();
        println!("{}", format!("{}", hex).bold().red());
    }


}


// /// Simple program to greet a person
// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
// struct Args {
//    /// Name of the person to greet
//    #[clap(short, long, value_parser)]
//    name: String,

//    /// Number of times to greet
//    #[clap(short, long, value_parser, default_value_t = 1)]
//    count: u8,
// }

// fn main() {
//    let args = Args::parse();

//    for _ in 0..args.count {
//        println!("Hello {}!", args.name)
//    }
// }
