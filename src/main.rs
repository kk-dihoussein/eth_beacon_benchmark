use anyhow::Result;
use eventsource::reqwest::Client;
use reqwest::{header, Error, Url};
use serde::Deserialize;
use tokio::time::{sleep, Duration};
use reqwest::{blocking::Client as ReqwestClient, header::HeaderMap};
use eventsource::reqwest::Client as EventSourceClient;
use tokio_stream::StreamExt;
use eventsource_client as es;

const BEACON_NODE_URL: &str = "https://crypto-stg-staking-service.azurewebsites.net";
const ACCESS_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6IllUY2VPNUlKeXlxUjZqekRTNWlBYnBlNDJKdyIsImtpZCI6IllUY2VPNUlKeXlxUjZqekRTNWlBYnBlNDJKdyJ9.eyJhdWQiOiJhcGk6Ly84ODA1M2M5My01NDg0LTQxZmMtOWY4OC0yNDI3ZjUxNzFjYWQiLCJpc3MiOiJodHRwczovL3N0cy53aW5kb3dzLm5ldC81MjRiNDljYy02Mjc0LTRlOTAtOWI4Yi0xYTA3MDE3YWRhOTEvIiwiaWF0IjoxNzM4MzE0NDM1LCJuYmYiOjE3MzgzMTQ0MzUsImV4cCI6MTczODMxOTgzMiwiYWNyIjoiMSIsImFpbyI6IkFWUUFxLzhaQUFBQWwxZGJrUStBd0ZQTVh3RjhBZk1nTVJRMDJWVThhVHJNSEcrMGQ4SStkb2FCMmZPOFc3Z0dDZlc5VXQrdVZKTjQ4NjFMZjZlQk05S3psM0pvajZYYk4zaTQvZXJjQ1kxOTliWnhjVjBOTmpvPSIsImFtciI6WyJwd2QiLCJtZmEiXSwiYXBwaWQiOiIwNGIwNzc5NS04ZGRiLTQ2MWEtYmJlZS0wMmY5ZTFiZjdiNDYiLCJhcHBpZGFjciI6IjAiLCJmYW1pbHlfbmFtZSI6IkRpYWIiLCJnaXZlbl9uYW1lIjoiSG91c3NlaW4iLCJpcGFkZHIiOiIyMDMuMTE0LjUwLjIxMCIsIm5hbWUiOiJIb3Vzc2VpbiBEaWFiIiwib2lkIjoiYzE5N2FiZDUtYjU3Zi00ZmYyLWExMjItNzg1YjQ2YjY3MzFkIiwicmgiOiIxLkFTc0F6RWxMVW5SaWtFNmJpeG9IQVhyYWtaTThCWWlFVlB4Qm40Z2tKX1VYSEswckFBWXJBQS4iLCJyb2xlcyI6WyJjcnlwdG8uc3Rha2luZy5mdW5jdGlvbi5hc3NpZ24iXSwic2NwIjoidXNlcl9pbXBlcnNvbmF0aW9uIiwic2lkIjoiNWM2MzIyNjYtNzM4Ny00MTQ2LWFhYzUtY2FjOWFlMDRjZDc1Iiwic3ViIjoiWUJxeHkzMGYxWHZJU3Z6alJNZ0xZQS0wdjd1VWdTcWk5a0lwN0UtZ1d3ZyIsInRpZCI6IjUyNGI0OWNjLTYyNzQtNGU5MC05YjhiLTFhMDcwMTdhZGE5MSIsInVuaXF1ZV9uYW1lIjoiaG91c3NlaW4uZGlhYkBkZXYuZnJ1aXRzLTFoYWRpdC5jb20iLCJ1cG4iOiJob3Vzc2Vpbi5kaWFiQGRldi5mcnVpdHMtMWhhZGl0LmNvbSIsInV0aSI6IkJSRjRTRjRJODBLUDZPR1lzUDltQUEiLCJ2ZXIiOiIxLjAifQ.cWZwySpIKwhvR1RaLZBzjFUUgJN0ELX-vDlmVKg7JpC4Jwd5LxZmJEoKtxH6AEhiyHUFmOUeLoeTiEt82M4LcKQEwy1EvhKDqZXDEptVeDPuJRAae0oz0DldzwNl_JQfPGo0K4TaaTs4dUtoWkh_9qKCFcv-682tTZ6jepE9c55SDtQp1WZPTvtYjtgatWvmgc3Dn0zU_ulhLVvTG1h_9tOnjYjlILv99xAHtDYZaPe-wd2dbOMiZL8KFecL3xDzPn9T6XtypCjGH97Tjir3QURHizvTz0UzTe_bOfxUE87nGVvvgugtkXJgq7JBOd6rXD2B_BuAusa1LB4bdsW7nQ";
#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(async {
        subscribe_to_head_events().await;
    });

    let current_state = fetch_current_state().await?;
    println!("Current state: {:?}", current_state);

    let mut slot = current_state.slot;
    while let Some(state) = fetch_historical_state(slot).await? {
        println!("Historical state at slot {}: {:?}", slot, state);
        slot -= 1;
        sleep(Duration::from_millis(100)).await; // Add delay to avoid rate limiting
    }

    Ok(())
}

async fn fetch_current_state() -> Result<State, Error> {
    let url = format!("{}/api/v1/beacon/states/head", BEACON_NODE_URL);
    let response = reqwest::get(&url).await?.json::<State>().await?;
    Ok(response)
}

async fn fetch_historical_state(slot: u64) -> Result<Option<State>> {
    let url = format!("{}/api/v1/beacon/states/{}", BEACON_NODE_URL, slot);
    let response = reqwest::get(&url).await?;
    let status = response.status();
    let body = response.text().await?;
    println!("fetch_historical_state response status: {}", status);
    println!("fetch_historical_state response body: {}", body);
    if status.is_success() {
        let state: State = serde_json::from_str(&body)?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

async fn subscribe_to_head_events() {
    let url = format!("{}/eth/v1/events?topics=head", BEACON_NODE_URL);

    let mut headers = header::HeaderMap::new();
    
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("Bearer {}", ACCESS_TOKEN)).unwrap());
    let mut client = es::ClientBuilder::for_url(&url).unwrap()
    .header("AUTHORIZATION", &format!("Bearer {}", ACCESS_TOKEN)).unwrap()
    .build();

    client
    .stream()
    .map_ok(|event| println!("got event: {:?}", event))
    .map_err(|err| eprintln!("error streaming events: {:?}", err));
}

#[derive(Debug, Deserialize)]
struct State {
    slot: u64,
}

#[derive(Debug, Deserialize)]
struct HeadEvent {
    slot: u64,
    // Add other fields as needed
}
