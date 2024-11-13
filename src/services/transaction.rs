use std::sync::Arc;
use std::error::Error;
use std::str::FromStr;
use std::{fs, io};
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Read, Write};
use ethers::core::types::{Address, TransactionRequest, U256, H256};
use ethers::core::types::transaction::eip2718::TypedTransaction;
use ethers::providers::{Http, Middleware, Provider, PendingTransaction};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{NameOrAddress, Signature};
use ethers::contract::Contract;
use ethers::abi::Abi;
use serde::{Deserialize, Serialize};
use crate::config::{STATE_FILE, STORAGE_DIR, ERC20_ABI};

#[derive(Serialize, Deserialize, Debug)]
struct StoredTransaction {
    #[serde(rename = "type")]
    tx_type: String,
    from: String,
    to: Option<String>,
    gas: String,
    gas_price: String,
    value: String,
    token_value: Option<String>,
}

impl StoredTransaction {
    fn from_address(&self) -> Option<Address> {
        Address::from_str(&self.from).ok()
    }

    fn to_address(&self) -> Option<Address> {
        self.to.as_deref().and_then(|to| Address::from_str(to).ok())
    }

    fn gas_as_u256(&self) -> U256 {
        U256::from_str_radix(&self.gas.trim_start_matches("0x"), 16).unwrap_or_default()
    }

    fn gas_price_as_u256(&self) -> U256 {
        U256::from_str_radix(&self.gas_price.trim_start_matches("0x"), 16).unwrap_or_default()
    }

    fn value_as_u256(&self) -> U256 {
        U256::from_str_radix(&self.value.trim_start_matches("0x"), 16).unwrap_or_default()
    }
}

impl StoredTransaction {
    fn from_typed_transaction(tx: &TypedTransaction) -> Self {
        let tx_type = match tx {
            TypedTransaction::Legacy(_) => "Legacy",
            TypedTransaction::Eip2930(_) => "Eip2930",
            TypedTransaction::Eip1559(_) => "Eip1559",
            _ => "Unknown",
        }
            .to_string();

        let from = tx.from().map(|addr| format!("{:?}", addr)).unwrap_or_default();

        let (to, token_value) = if let Some(input_data) = tx.data() {
            if input_data.len() >= 68 && input_data.starts_with(&[0xa9, 0x05, 0x9c, 0xbb]) {
                let recipient_address = Address::from_slice(&input_data[16..36]);
                let token_amount = U256::from_big_endian(&input_data[36..68]);
                (Some(format!("{:?}", recipient_address)), Some(token_amount.to_string()))
            } else {
                (tx.to().map(|to| format!("{:?}", to)), None)
            }
        } else {
            (tx.to().map(|to| format!("{:?}", to)), None)
        };

        StoredTransaction {
            tx_type,
            from,
            to,
            gas: tx.gas().map(|g| format!("{:#x}", g)).unwrap_or_default(),
            gas_price: tx.gas_price().map(|gp| format!("{:#x}", gp)).unwrap_or_default(),
            value: tx.value().map(|v| format!("{:#x}", v)).unwrap_or("0x0".to_string()),
            token_value,
        }
    }
}

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

    pub fn set_provider(&mut self, provider_url: &str) {
        let provider = Provider::<Http>::try_from(provider_url)
            .expect("Invalid provider URL");
        self.provider = Some(Arc::new(provider));
    }

    pub fn set_wallet(&mut self, wallet: LocalWallet) {
        self.wallet = Some(wallet);
    }

    pub async fn send(
        &mut self,
        to: &str,
        value: &str,
        gas_price: Option<&str>,
        gas_limit: Option<&str>,
        network_name: &str,
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

        self.save_history_to_file(&typed_tx, network_name);

        Ok(format!("{:#x}", tx_hash))
    }

    pub async fn send_token(
        &mut self,
        to: &str,
        value: &str,
        token_address: &str,
        gas_price: Option<&str>,
        gas_limit: Option<&str>,
        network_name: &str,
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

        self.save_history_to_file(&typed_tx, network_name);

        Ok(format!("{:#x}", tx_hash))
    }

    pub fn history(&self, network_name: &str) {
        let history = self.load_history_from_file(network_name);
        let account_name = Self::load_account_name().unwrap_or_default();

        if history.is_empty() {
            println!("No transaction history found for account {} on network {}", account_name, network_name);
        } else {
            println!("Transaction history for account '{}' on network '{}':", account_name, network_name);
            for (index, tx) in history.iter().enumerate() {
                println!("Transaction {}:", index + 1);
                println!("  From: {:?}", tx.from_address().unwrap_or(Address::zero()));
                println!("  To: {:?}", tx.to_address().unwrap_or(Address::zero()));
                println!("  Value: {:?}", tx.value_as_u256());
                println!("  Token Value: {:?}", tx.token_value.as_deref().and_then(|v| U256::from_dec_str(v).ok()).unwrap_or(U256::zero()));
                println!("  Gas Price: {:?}", tx.gas_price_as_u256());
                println!("  Gas Limit: {:?}", tx.gas_as_u256());
                println!("--------------------------------");
            }
        }
    }

    pub async fn info(&self, tx_hash: &str) -> Result<(), Box<dyn Error>> {
        let provider = self.provider.as_ref().ok_or("Provider not set")?;

        let hash: H256 = tx_hash.parse()?;
        let tx = provider.get_transaction(hash).await?;
        if let Some(transaction) = tx {
            println!("Transaction Info:");
            println!("  Hash: {:?}", transaction.hash);
            println!("  From: {:?}", transaction.from);

            if transaction.input.0.len() >= 68 && transaction.input.0.starts_with(&[0xa9, 0x05, 0x9c, 0xbb]) {
                let recipient_address = Address::from_slice(&transaction.input.0[16..36]);
                println!("  To: {:?}", recipient_address);
                Some(recipient_address)
            } else {
                println!("  To: {:?}", transaction.to.unwrap_or_default());
                transaction.to
            };

            println!("  Value: {:?}", transaction.value);

            if let Some(token_value) = transaction.input.0.get(36..68) {
                let token_amount = U256::from_big_endian(token_value);
                println!("  Token Value: {}", token_amount);
            } else {
                println!("  Token Value: Not applicable");
            }

            println!("  Gas Price: {:?}", transaction.gas_price.unwrap_or_default());
            println!("  Gas Limit: {:?}", transaction.gas);
            println!("  Nonce: {:?}", transaction.nonce);
            println!("  Block Hash: {:?}", transaction.block_hash.unwrap_or_default());
            println!("  Block Number: {:?}", transaction.block_number.unwrap_or_default());
            println!("  Transaction Index: {:?}", transaction.transaction_index.unwrap_or_default());
        } else {
            println!("Transaction not found for hash: {}", tx_hash);
        }
        Ok(())
    }


    pub async fn get_balance(&self, native_token: String) -> Result<(), Box<dyn Error>> {
        let wallet = self.wallet.as_ref().ok_or("Wallet not set")?;
        let provider = self.provider.as_ref().ok_or("Provider not set")?;
        println!("Wallet address: {:?}", wallet.address());
        let balance = provider.get_balance(wallet.address(), None).await?;
        let balance_eth = Self::wei_to_eth(balance);
        println!("Account balance: {} {}", balance_eth, native_token);
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

    fn tx_history_file(&self, network_name: &str) -> PathBuf {
        let account_name = Self::load_account_name().unwrap_or_default();
        Path::new(STORAGE_DIR)
            .join(account_name)
            .join(network_name)
            .join("tx_history.json")
    }

    fn load_history_from_file(&self, network_name: &str) -> Vec<StoredTransaction> {
        let path = self.tx_history_file(network_name);

        if !path.exists() {
            println!("Transaction history file does not exist at {:?}", path);
            return Vec::new();
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .expect("Failed to open transaction history file");
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).unwrap_or_else(|e| {
            println!("Deserialization error: {}", e);
            Vec::new()
        })
    }

    fn save_history_to_file(&self, tx: &TypedTransaction, network_name: &str) {
        let path = self.tx_history_file(network_name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create account directory");
        }

        let mut history = self.load_history_from_file(network_name);

        history.push(StoredTransaction::from_typed_transaction(tx));

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .expect("Failed to open transaction history file");
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, &history).expect("Failed to write transaction history to file");
    }
}
