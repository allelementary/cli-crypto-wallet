use std::sync::Arc;
use std::error::Error;
use std::str::FromStr;
use std::{fs, io};
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write};
use ethers::core::types::{Address, TransactionRequest, U256, H256};
use ethers::core::types::transaction::eip2718::TypedTransaction;
use ethers::providers::{Http, Middleware, Provider, PendingTransaction};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Signature;
use ethers::contract::Contract;
use ethers::abi::Abi;
use crate::config::{STATE_FILE, STORAGE_DIR, ERC20_ABI};

pub struct TransactionService {
    pub provider: Option<Arc<Provider<Http>>>,
    pub wallet: Option<LocalWallet>,
}

impl TransactionService {
    pub fn new() -> Self {
        TransactionService {
            provider: None,
            wallet: None,
        }
    }

    fn load_account_name() -> io::Result<String> {
        let state_path = Path::new(STATE_FILE);
        if !state_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "State file not found"));
        }

        let state_data = fs::read_to_string(state_path)?;
        let state_json: serde_json::Value = serde_json::from_str(&state_data)?;
        if let Some(account_name) = state_json["logged_in_account"].as_str() {
            Ok(account_name.to_string())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Account name not found in state file"))
        }
    }

    pub fn set_provider(&mut self, provider_url: &str) {
        let provider = Provider::<Http>::try_from(provider_url)
            .expect("Invalid provider URL");
        self.provider = Some(Arc::new(provider));
    }

    pub fn set_wallet(&mut self, wallet: LocalWallet) {
        self.wallet = Some(wallet);
    }

    fn tx_history_file(&self) -> PathBuf {
        let account_name = Self::load_account_name().unwrap_or_default();
        Path::new(STORAGE_DIR).join(account_name).join("tx_history.json")
    }

    fn load_history_from_file(&self) -> Vec<TypedTransaction> {
        let path = self.tx_history_file();

        if !path.exists() {
            return Vec::new();
        }

        let file = OpenOptions::new().read(true).open(&path).expect("Failed to open transaction history file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new()) // Return empty vector if deserialization fails
    }

    fn save_history_to_file(&self, tx: &TypedTransaction) {
        let path = self.tx_history_file();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create account directory");
        }

        let file = OpenOptions::new().create(true).append(true).open(&path).expect("Failed to open transaction history file");
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, tx).expect("Failed to write transaction to file");
        writer.write_all(b"\n").expect("Failed to write newline to file");
    }

    pub async fn send(
        &mut self,
        to: &str,
        value: &str,
        gas_price: Option<&str>,
        gas_limit: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let to_address = Address::from_str(to).map_err(|_| "Invalid destination address format")?;
        let value_in_wei = U256::from_dec_str(value).map_err(|_| "Invalid amount format")?;

        let provider = self.provider.as_ref().ok_or("Provider not set")?;
        let wallet = self.wallet.as_ref().ok_or("Wallet not set")?;

        let gas_price_in_wei = match gas_price {
            Some(gp) => U256::from_dec_str(gp).map_err(|_| "Invalid gas price format")?,
            None => provider.get_gas_price().await?,
        };

        let mut tx = TransactionRequest::pay(to_address, value_in_wei)
            .from(wallet.address())
            .gas_price(gas_price_in_wei);

        let mut typed_tx: TypedTransaction = tx.clone().into();

        let gas_limit_in_units = match gas_limit {
            Some(gl) => U256::from_dec_str(gl).map_err(|_| "Invalid gas limit format")?,
            None => provider.estimate_gas(&typed_tx, None).await?,
        };

        let nonce = provider
            .get_transaction_count(wallet.address(), None)
            .await?;

        let chain_id = provider.get_chainid().await?;
        typed_tx.set_chain_id(chain_id.as_u64());
        typed_tx.set_gas(gas_limit_in_units);
        typed_tx.set_nonce(nonce);

        let signature: Signature = wallet.sign_transaction(&typed_tx).await?;
        let signed_tx_bytes = typed_tx.rlp_signed(&signature);
        let pending_tx: PendingTransaction<'_, Http> = provider.send_raw_transaction(signed_tx_bytes).await?;
        let tx_hash = pending_tx.tx_hash();
        println!("Transaction sent. Hash: {:#x}", tx_hash);

        self.save_history_to_file(&typed_tx);

        Ok(format!("{:#x}", tx_hash))
    }

    pub async fn send_token(
        &mut self,
        to: &str,
        value: &str,
        token_address: &str,
        gas_price: Option<&str>,
        gas_limit: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let to_address = Address::from_str(to).map_err(|_| "Invalid destination address format")?;
        let value_in_wei = U256::from_dec_str(value).map_err(|_| "Invalid amount format")?;

        let provider = self.provider.as_ref().ok_or("Provider not set")?;
        let wallet = self.wallet.as_ref().ok_or("Wallet not set")?;

        let gas_price_in_wei = match gas_price {
            Some(gp) => U256::from_dec_str(gp).map_err(|_| "Invalid gas price format")?,
            None => provider.get_gas_price().await?,
        };

        let token_address = Address::from_str(token_address).map_err(|_| "Invalid token address format")?;
        let abi: Abi = serde_json::from_str(ERC20_ABI)?;
        let contract = Contract::new(token_address, abi, provider.clone());
        let tx = contract.method::<(Address, U256), bool>("transfer", (to_address, value_in_wei))?
            .from(wallet.address())
            .gas_price(gas_price_in_wei);

        let mut tx_request = tx.tx;

        let gas_limit_in_units = match gas_limit {
            Some(gl) => U256::from_dec_str(gl).map_err(|_| "Invalid gas limit format")?,
            None => provider.estimate_gas(&tx_request, None).await?,
        };

        let nonce = provider
            .get_transaction_count(wallet.address(), None)
            .await?;

        let chain_id = provider.get_chainid().await?;
        tx_request.set_chain_id(chain_id.as_u64());
        tx_request.set_gas(gas_limit_in_units);
        tx_request.set_nonce(nonce);

        let typed_tx: TypedTransaction = tx_request.into();

        let signature = wallet.sign_transaction(&typed_tx).await?;
        let signed_tx_bytes = typed_tx.rlp_signed(&signature);
        let pending_tx: PendingTransaction<'_, Http> = provider.send_raw_transaction(signed_tx_bytes).await?;

        let tx_hash = pending_tx.tx_hash();
        println!("Token transfer sent. Hash: {:#x}", tx_hash);

        Ok(format!("{:#x}", tx_hash))
    }

    pub fn history(&self) {
        let history = self.load_history_from_file();
        let account_name = Self::load_account_name().unwrap_or_default();

        if history.is_empty() {
            println!("No transaction history found for this account.");
        } else {
            println!("Transaction history for account '{}':", account_name);
            for (index, tx) in history.iter().enumerate() {
                println!("Transaction {}:", index + 1);
                println!("  To: {:?}", tx.to());
                println!("  Value: {:?}", tx.value());
                println!("  Gas Price: {:?}", tx.gas_price());
                println!("  Gas Limit: {:?}", tx.gas());
                println!("  Data: {:?}", tx.data());
                println!("--------------------------------");
            }
        }
    }

    pub async fn info(&self, tx_hash: &str) -> Result<(), Box<dyn Error>> {
        let provider = self.provider.as_ref().ok_or("Provider not set")?;

        let hash: H256 = tx_hash.parse()?;
        let tx = provider.get_transaction(hash).await?;
        if let Some(transaction) = tx {
            println!("Transaction Info: {:?}", transaction);
        } else {
            println!("Transaction not found for hash: {}", tx_hash);
        }
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<(), Box<dyn Error>> {
        let wallet = self.wallet.as_ref().ok_or("Wallet not set")?;
        let provider = self.provider.as_ref().ok_or("Provider not set")?;
        println!("Wallet address: {:?}", wallet.address());
        let balance = provider.get_balance(wallet.address(), None).await?;
        let balance_eth = Self::wei_to_eth(balance);
        // todo: replace ETH with network native token (get from current network)
        println!("Account balance: {} ETH", balance_eth);
        Ok(())
    }

    pub async fn get_token_balance(
        &self,
        token_address: &str,
    ) -> Result<U256, Box<dyn Error>> {
        let token_address = Address::from_str(token_address).map_err(|_| "Invalid token address format")?;
        let wallet = self.wallet.as_ref().ok_or("Wallet not set")?;
        let provider = self.provider.as_ref().ok_or("Provider not set")?;
        let abi: Abi = serde_json::from_str(ERC20_ABI)?;
        let contract = Contract::new(token_address, abi, provider.clone());
        let balance: U256 = contract
            .method::<_, U256>("balanceOf", wallet.address())?
            .call()
            .await?;
        println!("Account balance: {} {:?}", balance, token_address);
        Ok(balance)
    }

    fn wei_to_eth(wei: U256) -> String {
        let eth_in_wei = U256::exp10(18);
        let eth = wei / eth_in_wei;
        let remainder = wei % eth_in_wei;
        format!("{}.{}", eth, remainder)
    }
}
