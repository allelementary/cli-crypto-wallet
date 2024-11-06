use clap::{Parser, Subcommand};
mod services;

#[derive(Parser)]
#[command(
    name = "vault-cli",
    version = "1.0",
    author = "Mikhail Antonov <allelementaryfor@gmail.com>",
    about = "CLI Crypto Wallet Application"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
enum AccountCommands {
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
enum NetworkCommands {
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
}

#[derive(Subcommand)]
enum TxCommands {
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

fn main() {
    let cli = Cli::parse();

    let mut network_service = services::network::NetworkService::new();

    match &cli.command {
        Commands::Account { subcommand } => match subcommand {
            AccountCommands::Create { account_name } => {
                services::account::AccountService::create_account(account_name);
            }
            AccountCommands::Login { account_name } => {
                services::account::AccountService::login(account_name);
            }
            AccountCommands::List => {
                services::account::AccountService::list();
            }
            AccountCommands::Logout => {
                services::account::AccountService::logout();
            }
            AccountCommands::Balance => {
                println!("Account balance called");
            }
            AccountCommands::Info => {
                services::account::AccountService::account_info();
            }
        },
        Commands::Network { subcommand } => match subcommand {
            NetworkCommands::Switch { network_name, url } => {
                network_service.switch_network(network_name, url.as_deref());
            }
            NetworkCommands::List => {
                network_service.list_networks();
            }
            NetworkCommands::Add { network_name, rpc_url, native_token, chain_id } => {
                network_service.add_network(network_name, rpc_url, native_token, *chain_id);
            }
        },
        Commands::Tx { subcommand } => match subcommand {
            TxCommands::Send {
                amount,
                destination_address,
                gas_price,
                gas_limit,
            } => {
                println!(
                    "Transaction send called with amount: {}, destination: {}",
                    amount, destination_address
                );
                if let Some(price) = gas_price {
                    println!("Gas price specified: {}", price);
                }
                if let Some(limit) = gas_limit {
                    println!("Gas limit specified: {}", limit);
                }
            }
            TxCommands::History => {
                println!("Transaction history called");
            }
            TxCommands::Info { transaction_hash } => {
                println!("Transaction info called with hash: {}", transaction_hash);
            }
        },
    }
}
