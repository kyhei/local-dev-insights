mod db;
mod mcp;
mod prompts;
mod resources;
mod server;
mod tools;

use anyhow::Result;
use dotenv::dotenv;
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Initialize logging if needed, strictly to stderr
    eprintln!("Local Dev Insights MCP Server starting...");

    let server = Server::new().await?;
    server.run().await?;

    Ok(())
}
