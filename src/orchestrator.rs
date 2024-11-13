use crate::commands::{Commands, AccountCommands, NetworkCommands, TxCommands};
use super::services::{account::AccountService, network::NetworkService, transaction::TransactionService};

pub struct Orchestrator {
    account_service: AccountService,
    network_service: NetworkService,
    transaction_service: TransactionService,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            account_service: AccountService,
            network_service: NetworkService::new(),
            transaction_service: TransactionService::new(),
        }
    }

    pub async fn handle_command(&mut self, command: &Commands) {
        if let Some(wallet) = AccountService::get_wallet() {
            self.transaction_service.set_wallet(wallet);
        } else {
            eprintln!("Warning: Wallet not set. Please log in or create a wallet");
        }

        if let Some(provider_url) = self.network_service.get_provider_url() {
            self.transaction_service.set_provider(provider_url.as_str());
        } else {
            eprintln!("Warning: Provider URL not set. Some commands may not work until the provider is set.");
        }

        match command {
            Commands::Account { subcommand } => {
                self.handle_account_commands(subcommand).await;
            }
            Commands::Network { subcommand } => {
                self.handle_network_commands(subcommand);
            }
            Commands::Tx { subcommand } => {
                self.handle_tx_commands(subcommand).await;
            }
        }
    }

    pub async fn handle_account_commands(&mut self, command: &AccountCommands) {
        match command {
            AccountCommands::Create { account_name } => {
                AccountService::create_account(account_name);
            }
            AccountCommands::Login { account_name } => {
                AccountService::login(account_name);
            }
            AccountCommands::List => {
                AccountService::list();
            }
            AccountCommands::Logout => {
                AccountService::logout();
            }
            AccountCommands::Balance => {
                let native_token = self.network_service.get_native_token();
                if let Err(e) = self.transaction_service.get_balance(native_token.unwrap()).await {
                    eprintln!("Failed to retrieve balance: {}", e);
                }
            }
            AccountCommands::BalanceToken { token_address } => {
                if let Err(e) = self.transaction_service.get_token_balance(token_address).await {
                    eprintln!("Failed to retrieve token balance: {}", e);
                }
            }
            AccountCommands::Info => {
                AccountService::account_info();
            }
        }
    }

    pub fn handle_network_commands(&mut self, command: &NetworkCommands) {
        match command {
            NetworkCommands::Switch { network_name, url } => {
                self.network_service.switch_network(network_name, url.as_deref());
            }
            NetworkCommands::List => {
                self.network_service.list_networks();
            }
            NetworkCommands::Add { network_name, rpc_url, native_token, chain_id } => {
                self.network_service.add_network(network_name, rpc_url, native_token, *chain_id);
            }
            NetworkCommands::SetUrl { network_name, url } => {
                self.network_service.set_network_url(network_name, url);
            }
            NetworkCommands::Info => {
                self.network_service.network_info();
            }
        }
    }

    pub async fn handle_tx_commands(&mut self, command: &TxCommands) {
        match command {
            TxCommands::Send {
                amount,
                destination_address,
                gas_price,
                gas_limit,
            } => {
                match self.transaction_service.send(
                    destination_address, amount, gas_price.as_deref(), gas_limit.as_deref()
                ).await {
                    Ok(tx_hash) => println!("Transaction sent successfully. Hash: {}", tx_hash),
                    Err(e) => println!("Failed to send transaction: {}", e),
                }
            }
            TxCommands::SendToken {
                amount,
                destination_address,
                token_address,
                gas_price,
                gas_limit,
            } => {
                match self.transaction_service.send_token(
                    destination_address, amount, token_address, gas_price.as_deref(), gas_limit.as_deref()
                ).await {
                    Ok(tx_hash) => println!("Transaction sent successfully. Hash: {}", tx_hash),
                    Err(e) => println!("Failed to send transaction: {}", e),
                }
            }
            TxCommands::History => {
                self.transaction_service.history();
            }
            TxCommands::Info { transaction_hash } => {
                match self.transaction_service.info(transaction_hash).await {
                    Ok(_) => println!("Transaction info retrieved successfully."),
                    Err(e) => println!("Failed to retrieve transaction info: {}", e),
                }
            }
        }
    }
}
