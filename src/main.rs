use crate::starknet_client::StarkNetChain;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[macro_use]
extern crate rocket;

mod github;
mod rest;
mod starknet_client;

#[launch]
fn rocket() -> _ {
    info!("loading configuration...");

    let github_id = std::env::var("GITHUB_ID").expect("GITHUB_ID environment variable must be set");
    let github_secret =
        std::env::var("GITHUB_SECRET").expect("GITHUB_SECRET environment variable must be set");
    let access_token_url = std::env::var("GITHUB_ACCESS_TOKEN_URL")
        .unwrap_or_else(|_| "https://github.com/login/oauth/access_token".to_string());
    let user_api_url = std::env::var("GITHUB_USER_API_URL")
        .unwrap_or_else(|_| "https://api.github.com/user".to_string());

    let hex_account_address = std::env::var("STARKNET_ACCOUNT")
        .expect("STARKNET_ACCOUNT environment variable must be set");
    let hex_private_key = std::env::var("STARKNET_PRIVATE_KEY")
        .expect("STARKNET_PRIVATE_KEY environment variable must be set");
    let hex_badge_registry_address = std::env::var("STARKNET_BADGE_REGISTRY_ADDRESS")
        .expect("STARKNET_BADGE_REGISTRY_ADDRESS environment variable must be set");
    let chain = std::env::var("STARKNET_CHAIN")
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");
    let chain: StarkNetChain = chain
        .parse()
        .expect("STARKNET_CHAIN environment variable must be set to either 'MAINNET' or 'TESTNET'");

    info!("configuration loaded");

    let github_client =
        github::GitHubClient::new(github_id, github_secret, access_token_url, user_api_url);

    let starknet_client = starknet_client::StarkNetClient::new(
        &hex_account_address,
        &hex_private_key,
        &hex_badge_registry_address,
        chain,
    );

    rocket::build()
        .manage(github_client)
        .manage(starknet_client)
        .attach(CORS)
        .mount("/", routes![rest::health_check])
        .mount(
            "/registrations",
            routes![options_handler, rest::register_github_user],
        )
}

#[route(OPTIONS, uri = "/github")]
fn options_handler() -> Status {
    Status::Ok
}
