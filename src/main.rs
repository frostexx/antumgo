use log::info;
use warp::Filter;

mod app;
mod components;
mod models;
mod services;
mod workers;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Pi Sweeper Bot Pro");

    // The server part is now primarily for serving the frontend
    // and handling WebSocket connections for live logging.
    let static_files = warp::fs::dir("static");

    let routes = static_files.or(warp::path::end().map(|| warp::fs::file("static/index.html")));

    // In a real-world scenario, you might have API endpoints here,
    // but we are moving logic to workers triggered from the frontend.
    // For simplicity, we'll let the frontend handle most logic via wasm.

    info!("Serving static files from 'static/' directory");
    info!("Access the bot at http://127.0.0.1:3030");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}