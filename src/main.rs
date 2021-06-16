use crud_toy::env_var;
use dotenv::dotenv;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    let addr: SocketAddr = env_var!("SERVER_ADDR")
        .parse()
        .expect("unable to parse socket address");
    crud_toy::route::run(addr).await
}
