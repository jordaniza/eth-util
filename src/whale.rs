use ethers::{prelude::*, utils::Anvil};
use futures::future::join_all;
use futures::Future;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::convert::TryFrom;
use std::env;
use std::str::FromStr;
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Holder {
    address: String,
    balance: f64,
    share: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct HolderBalance {
    holder: Holder,
    eth: U256,
}

#[derive(Serialize, Deserialize, Debug)]
struct Res {
    holders: Vec<Holder>,
}

async fn fetch_whales(token: String) -> Res {
    let api_key = env::var("ETHPLORER_API_KEY").expect("Missing API Key");

    let client = reqwest::Client::new();
    let request_url = format!(
        "https://api.ethplorer.io/getTopTokenHolders/{}?apiKey={}&limit={limit}",
        token,
        api_key,
        limit = "30"
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

async fn get_balance(holder: Holder, provider: &Provider<Http>) -> HolderBalance {
    println!("Fetching holder {}", holder.address);
    let account = H160::from_str(&holder.address).unwrap();
    let balance = provider.get_balance(account, None);
    return HolderBalance {
        eth: balance.await.unwrap(),
        holder,
    };
}

async fn _get_balances(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    let mut new_data = Vec::new();
    for h in data.holders {
        let new_holder = get_balance(h, &provider).await;
        new_data.push(new_holder);
    }
    return new_data;
}

type AsyncFunction<R> = fn(Res, Provider<Http>) -> R;

async fn time_it<T>(
    data: Res,
    provider: Provider<Http>,
    inner: AsyncFunction<impl Future<Output = T>>,
) -> T {
    let start = Instant::now();
    let new_data: T = inner(data, provider).await;
    print!("Completed in {}ms", start.elapsed().as_millis());
    return new_data;
}

async fn _get_balances_concurrent(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    let mut new_data = Vec::new();
    let mut futures_vec = Vec::new();
    for h in data.holders {
        futures_vec.push(get_balance(h, &provider));
    }
    new_data = join_all(futures_vec).await;
    return new_data;
}

async fn get_balances(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    return time_it(data, provider, _get_balances).await;
}

async fn get_balances_concurrent<'a>(data: Res, provider: Provider<Http>) -> Vec<HolderBalance> {
    return time_it(data, provider, _get_balances_concurrent).await;
}

pub async fn get_whale_balances_for(token: String) {
    let provider = Provider::<Http>::try_from(String::from("https://rpc.ankr.com/eth")).unwrap();

    let data = fetch_whales(token).await;
    // let json_data = get_balances(data, provider).await;
    let json_data = get_balances_concurrent(data, provider).await;

    let output_path = "balances.json".to_string();
    std::fs::write(output_path, to_string_pretty(&json_data).unwrap()).unwrap();
}
