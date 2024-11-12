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
    /// Account management commands
    Account {
        #[command(subcommand)]
        subcommand: AccountCommands,
    },
    /// Network management commands
    Network {
        #[command(subcommand)]
        subcommand: NetworkCommands,
    },
    /// Transaction-related commands
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
    /// Send a transaction
    Send {
        /// Amount to send
        amount: String,
        /// Destination address
        destination_address: String,
        /// Gas price (optional)
        #[arg(long)]
        gas_price: Option<String>,
        /// Gas limit (optional)
        #[arg(long)]
        gas_limit: Option<String>,
    },
    /// View transaction history
    History,
    /// Get transaction details
    Info {
        /// Transaction hash
        transaction_hash: String,
    },
}
