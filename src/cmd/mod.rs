use clap::Parser;

pub mod server;

use server::Server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct MointerCommand {
    #[command(subcommand)]
    subcommand: SubCommand,
}

impl MointerCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        self.subcommand.run().await
    }
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command(name = "server")]
    Server(Server),
}

impl SubCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            SubCommand::Server(server) => server.run().await,
        }
    }
}
