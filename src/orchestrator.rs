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
        let wallet = AccountService::get_wallet();
        let provider_url = self.network_service.get_provider_url();
        self.transaction_service.set_provider(provider_url.unwrap().as_str());
        self.transaction_service.set_wallet(wallet.unwrap());

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
                if let Err(e) = self.transaction_service.get_balance().await {
                    eprintln!("Failed to retrieve balance: {}", e);
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
        // todo: add send token <token-address>
        //  at the moment it sends only native
        //  - add get token balance
        //  How to get all wallet tokens balances?
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
