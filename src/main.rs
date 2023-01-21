use controller::http::start_http_server;

pub(crate) mod controller;
pub(crate) mod usecase;
pub(crate) mod util;

#[tokio::main]
async fn main() {
    start_http_server().await.unwrap();
}
