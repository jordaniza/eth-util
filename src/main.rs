use rand::Rng;
use std::fmt;


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
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    println!("------- ⟠⟠⟠ WΞlcomΞ to Ξth Util ⟠⟠⟠ ----------");
    println!("-------             {}           ----------", VERSION);

    let hex = generate_hex_address();

    println!("{}", hex);

}
