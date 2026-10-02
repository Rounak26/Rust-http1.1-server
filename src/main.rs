use rust_http_server::config::Config;
use rust_http_server::error::Result;
use rust_http_server::handler;
use rust_http_server::router::Router;
use rust_http_server::server::Server;

fn main() {
    if let Err(err) = run() {
        eprintln!("fatal: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::from_env()?;

    let mut router = Router::new();
    router
        .get("/", handler::index)
        .get("/health", handler::health)
        .get("/hello/:name", handler::hello)
        .get("/query", handler::echo_query)
        .post("/echo", handler::echo);

    Server::new(config, router).run()
}
