use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vault-cli",
    version = "1.0",
    author = "Mikhail Antonov <allelementaryfor@gmail.com>",
    about = "CLI Crypto Wallet Application"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Account {
        #[command(subcommand)]
        subcommand: AccountCommands,
    },
    Network {
        #[command(subcommand)]
        subcommand: NetworkCommands,
    },
    Tx {
        #[command(subcommand)]
        subcommand: TxCommands,
    },
}

#[derive(Subcommand)]
pub enum AccountCommands {
    Create {
        account_name: String,
    },
    Login {
        account_name: String,
    },
    List,
    Logout,
    Balance,
    BalanceToken {
        token_address: String,
    },
    Info,
}

#[derive(Subcommand)]
pub enum NetworkCommands {
    Switch {
        network_name: String,
        url: Option<String>,
    },
    List,
    Add {
        network_name: String,
        #[arg(long)]
        rpc_url: String,
        native_token: String,
        chain_id: u64,
    },
    SetUrl {
        network_name: String,
        url: String,
    },
    Info,
}

#[derive(Subcommand)]
pub enum TxCommands {
    Send {
        amount: String,
        destination_address: String,
        #[arg(long)]
        gas_price: Option<String>,
        #[arg(long)]
        gas_limit: Option<String>,
    },
    SendToken {
        amount: String,
        destination_address: String,
        token_address: String,
        #[arg(long)]
        gas_price: Option<String>,
        #[arg(long)]
        gas_limit: Option<String>,
    },
    History,
    Info {
        transaction_hash: String,
    },
}
