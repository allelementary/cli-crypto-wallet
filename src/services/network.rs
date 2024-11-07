use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use serde_json::{json, Value};

const STORAGE_FILE: &str = "storage/networks.json";

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub url: Option<String>,
    pub native_token: String,
    pub chain_id: u64,
}

pub struct NetworkService {
    pub networks: HashMap<String, NetworkInfo>,
    pub current_network: Option<String>,
}

impl NetworkService {
    pub fn new() -> Self {
        let mut networks = HashMap::new();

        networks.insert(
            "ethereum_mainnet".to_string(),
            NetworkInfo {
                name: "Ethereum Mainnet".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 1,
            },
        );

        networks.insert(
            "ethereum_sepolia".to_string(),
            NetworkInfo {
                name: "Ethereum Sepolia".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 11155111,
            },
        );

        networks.insert(
            "polygon_mainnet".to_string(),
            NetworkInfo {
                name: "Polygon Mainnet".to_string(),
                url: None,
                native_token: "POL".to_string(),
                chain_id: 137,
            },
        );

        networks.insert(
            "polygon_amoy".to_string(),
            NetworkInfo {
                name: "Polygon Amoy".to_string(),
                url: None,
                native_token: "POL".to_string(),
                chain_id: 80002,
            },
        );

        networks.insert(
            "optimism_mainnet".to_string(),
            NetworkInfo {
                name: "Optimism Mainnet".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 10,
            },
        );

        networks.insert(
            "optimism_sepolia".to_string(),
            NetworkInfo {
                name: "Optimism Sepolia".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 11155420,
            },
        );

        networks.insert(
            "bsc_mainnet".to_string(),
            NetworkInfo {
                name: "Binance Smart Chain Mainnet".to_string(),
                url: None,
                native_token: "BNB".to_string(),
                chain_id: 56,
            },
        );

        networks.insert(
            "bsc_testnet".to_string(),
            NetworkInfo {
                name: "Binance Smart Chain Testnet".to_string(),
                url: None,
                native_token: "BNB".to_string(),
                chain_id: 97,
            },
        );

        networks.insert(
            "arbitrum_mainnet".to_string(),
            NetworkInfo {
                name: "Arbitrum Mainnet".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 42161,
            },
        );

        networks.insert(
            "arbitrum_sepolia".to_string(),
            NetworkInfo {
                name: "Arbitrum Sepolia".to_string(),
                url: None,
                native_token: "ETH".to_string(),
                chain_id: 421614,
            },
        );

        let mut service = NetworkService {
            networks,
            current_network: None,
        };

        service.load_state();
        service
    }

    pub fn set_network_url(&mut self, network_name: &str, url: &str) {
        if let Some(network) = self.networks.get_mut(network_name) {
            network.url = Some(url.to_string());
            println!("URL for '{}' has been set to '{}'.", network_name, url);
            self.save_state();
        } else {
            println!("Network '{}' not found.", network_name);
        }
    }

    pub fn get_network(&self, network_name: &str) -> Option<&NetworkInfo> {
        self.networks.get(network_name)
    }

    pub fn switch_network(&mut self, network_name: &str, url: Option<&str>) {
        if let Some(network) = self.networks.get_mut(network_name) {
            if let Some(url) = url {
                network.url = Some(url.to_string());
                println!("Switched to network '{}'. URL set to '{}'.", network_name, url);
            } else if network.url.is_some() {
                println!("Switched to network '{}'. Using existing URL: '{}'.", network_name, network.url.as_ref().unwrap());
            } else {
                println!("Network '{}' requires a valid URL. Please provide one.", network_name);
            }
            self.current_network = Some(network_name.to_string());
            self.save_state();
        } else {
            println!("Network '{}' not found.", network_name);
        }
    }

    pub fn prompt_for_url(&mut self, network_name: &str) {
        print!("Enter the RPC URL for '{}': ", network_name);
        io::stdout().flush().unwrap();
        let mut url = String::new();
        io::stdin().read_line(&mut url).expect("Failed to read input");
        let url = url.trim();

        if !url.is_empty() {
            self.set_network_url(network_name, url);
        } else {
            println!("No URL provided for '{}'.", network_name);
        }
    }

    pub fn list_networks(&self) {
        println!("Available networks:");
        for (key, network) in &self.networks {
            println!(
                "- {}: {} (Chain ID: {}, URL: {:?})",
                key, network.name, network.chain_id, network.url.as_deref().unwrap_or("None")
            );
        }
    }

    pub fn add_network(&mut self, network_name: &str, url: &str, native_token: &str, chain_id: u64) {
        if self.networks.values().any(|network| network.chain_id == chain_id) {
            println!("Network with chain ID '{}' already exists.", chain_id);
        } else if self.networks.contains_key(&network_name.to_lowercase()) {
            println!("Network '{}' already exists.", network_name);
        } else {
            self.networks.insert(
                network_name.to_string(),
                NetworkInfo {
                    name: network_name.to_string(),
                    url: Some(url.to_string()),
                    native_token: native_token.to_string(),
                    chain_id,
                },
            );
            println!("Network '{}' added successfully.", network_name);
            self.save_state();
        }
    }

    pub fn save_state(&self) {
        let state = json!({
            "current_network": self.current_network,
            "networks": self.networks.iter().map(|(key, value)| {
                (key, json!({
                    "name": value.name,
                    "url": value.url,
                    "native_token": value.native_token,
                    "chain_id": value.chain_id,
                }))
            }).collect::<HashMap<_, _>>()
        });

        if let Err(e) = fs::create_dir_all("storage") {
            println!("Failed to create storage directory: {}", e);
            return;
        }

        if let Err(e) = fs::write(STORAGE_FILE, state.to_string()) {
            println!("Failed to save network state: {}", e);
        }
    }

    pub fn load_state(&mut self) {
        let state_data = match fs::read_to_string(STORAGE_FILE) {
            Ok(data) => data,
            Err(_) => return,
        };

        let state_json: Value = match serde_json::from_str(&state_data) {
            Ok(json) => json,
            Err(_) => return,
        };

        if let Some(current_network) = state_json["current_network"].as_str() {
            self.current_network = Some(current_network.to_string());
        }

        if let Some(networks) = state_json["networks"].as_object() {
            for (key, value) in networks {
                let network_info = NetworkInfo {
                    name: value["name"].as_str().unwrap_or_default().to_string(),
                    url: value["url"].as_str().map(|s| s.to_string()),
                    native_token: value["native_token"].as_str().unwrap_or_default().to_string(),
                    chain_id: value["chain_id"].as_u64().unwrap_or_default(),
                };
                self.networks.insert(key.clone(), network_info);
            }
        }
    }
}
