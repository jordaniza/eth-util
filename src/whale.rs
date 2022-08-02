use ethers::prelude::*;
use futures::future::{join, join_all};
use futures::Future;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::convert::TryFrom;
use std::env;
use std::str::FromStr;
use std::time::Instant;

const RPC_URL: &str = "https://rpc.ankr.com/eth";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Holder {
    address: String,
    balance: f64,
    share: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Token {
    address: String,                  // token address,
    total_supply: String,             // total token supply,
    name: String,                     // token name,
    symbol: String,                   // token symbol,
    decimals: String,                 // number of significant digits,
    holders_count: f64,               // total numnber of token holders,
    public_tags: Option<Vec<String>>, // [optional] one or more tags from https://ethplorer.io/tag/,
    owner: Option<String>,            // token owner address,
    count_ops: Option<f64>,           // total count of token operations,
    total_in: Option<f64>,            // total amount of incoming tokens,
    total_out: Option<f64>,           // total amount of outgoing tokens,
    transfers_count: Option<f64>,     // total number of token operations,
    eth_transfers_count: Option<f64>, // [optional] total number of ethereum operations,
    issuances_count: Option<f64>,     // total count of token issuances,
    image: Option<String>,            // [optional] token image url,
    description: Option<String>,      // [optional] token description,
    website: Option<String>,          // [optional] token website url,
    last_updated: Option<usize>,      // last updated timestamp,
    price: TokenPrice,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TokenPrice {
    rate: f64,             // current price in currency,
    currency: String,      // token price currency (USD),
    diff: f64,             // 24 hours rate difference (in percent),
    diff7d: f64,           // 7 days rate difference (in percent),
    diff30d: Option<f64>,  // 30 days rate difference (in percent),
    market_cap_usd: f64,   // market cap (USD),
    available_supply: f64, // available supply,
    volume24h: f64,        // 24 hours volume,
    ts: f64,               // last rate update timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
struct HolderBalance {
    holder: Holder,
    eth: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Res {
    holders: Vec<Holder>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    data: Vec<HolderBalance>,
    token: String,
    name: String,
    symbol: String,
    price: f64,
    holders: f64,
}

/// Fetch current list of 🐋s for a given token
/// There is no guarantee these guys will have eth
async fn fetch_whales(token: &String, number: u8) -> Res {
    let api_key = env::var("ETHPLORER_API_KEY").expect("Missing API Key - Set the environment variable \"ETHPLORER_API_KEY\" or run export ETHPLORER_API_KEY={MY_KEY}");

    let client = reqwest::Client::new();
    let request_url = format!(
        "https://api.ethplorer.io/getTopTokenHolders/{}?apiKey={}&limit={}",
        token, api_key, number
    );
    println!("{}", request_url);
    let res = client
        .get(request_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "reqwest/0.11.11")
        .send()
        .await;

    return match res {
        Ok(data) => data.json::<Res>().await.unwrap(),
        _ => panic!("Error fetching response"),
    };
}

async fn fetch_token_metadata(address: &String) -> Token {
    let api_key = env::var("ETHPLORER_API_KEY").expect("Missing API Key");

    let client = reqwest::Client::new();
    let request_url = format!(
        "https://api.ethplorer.io/getTokenInfo/{address}?apiKey={}",
        api_key
    );
    println!("{}", request_url);
    let res = client
        .get(request_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "reqwest/0.11.11")
        .send()
        .await;

    return match res {
        Ok(data) => data.json::<Token>().await.unwrap(),
        _ => panic!("Error fetching token"),
    };
}

async fn get_balance(holder: Holder, provider: &Provider<Http>) -> HolderBalance {
    println!("Fetching holder {}", holder.address);
    let account = H160::from_str(&holder.address).unwrap();
    let balance = provider.get_balance(account, None);
    return HolderBalance {
        eth: balance.await.unwrap().to_string(),
        holder,
    };
}

/// old implementation in sequence, this is pretty slow
async fn _get_balances(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    let mut new_data = Vec::new();
    for h in data.holders {
        let new_holder = get_balance(h, &provider).await;
        new_data.push(new_holder);
    }
    return new_data;
}

/// Brittle decorator: will only work for a very strict function
type HOF<R> = fn(Res, Provider<Http>) -> R;
async fn time_it<T>(data: Res, provider: Provider<Http>, inner: HOF<impl Future<Output = T>>) -> T {
    let start = Instant::now();
    let new_data: T = inner(data, provider).await;
    print!("Completed in {}ms\n", start.elapsed().as_millis());
    return new_data;
}

/// Executes RPC calls to the provider simultaneously using futures.
/// This fetches the balance of the whale in eth.
/// As these are on-chain calls it would be better to route through multicall
/// Speed improvement is not as much as I expected?
async fn _get_balances_concurrent(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    let mut futures_vec = Vec::new();
    for h in data.holders {
        futures_vec.push(get_balance(h, &provider));
    }
    // execute in parrallel
    return join_all(futures_vec).await;
}

async fn _get_balances_concurrent_it(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    let futures_vec = data
        .holders
        .into_iter()
        .map(|h: Holder| return get_balance(h, &provider));

    return join_all(futures_vec).await;
}

async fn get_balances_concurrent(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    return time_it(data, provider, _get_balances_concurrent).await;
}

pub async fn get_whale_balances_for(token: String, number: u8) -> Output {
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();

    // batch first set of requests
    let (whales, token_metadata) =
        join(fetch_whales(&token, number), fetch_token_metadata(&token)).await;

    let data: Vec<HolderBalance> = get_balances_concurrent(whales, provider).await;

    return Output {
        data,
        token,
        name: token_metadata.name,
        price: token_metadata.price.rate,
        symbol: token_metadata.symbol,
        holders: token_metadata.holders_count,
    };
}

pub fn write_to_file(json_data: &Output, output_path: String) {
    let contents = to_string_pretty(json_data).unwrap();
    std::fs::write(&output_path, contents).unwrap();
    print!("Wrote contents to file \"{}\"\n", output_path);
}
