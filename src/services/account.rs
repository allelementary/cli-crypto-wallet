/*
    AccountService - responsible for account management operations.
    - Create new accounts, including password setup and seed phrase generation.
    - Store account information securely using encryption.
    - List available accounts, perform login/logout, etc.
*/
use std::fs;
use std::io::{self, Write};
use bip39::{Mnemonic, Language};
use ethers::prelude::*;
use serde_json::{json, Value};
use aes_gcm::{
    aead::{Aead, KeyInit, Nonce},
    Aes256Gcm, Key
};
use crate::services::crypto::CryptoService;

const STORAGE_DIR: &str = "storage";
const STATE_FILE: &str = "storage/state.json";

pub struct AccountService;

impl AccountService {
    pub fn create_account(account_name: &str) {
        let password = AccountService::get_password("Set a password: ");
        let password_confirmation = AccountService::get_password("Enter the password again for confirmation: ");

        if password != password_confirmation {
            println!("Passwords do not match. Please try again.");
            return;
        }

        let mnemonic = Mnemonic::generate_in(Language::English, 12).expect("Failed to generate mnemonic");
        let seed_phrase = mnemonic.to_string();

        println!("Your wallet has been created. Please write down the following seed phrase on a piece of paper as a backup:");
        println!("{}", seed_phrase);

        let crypto_service = CryptoService{};
        let encryption_key = CryptoService::generate_key();

        let encrypted_seed = match crypto_service.encrypt(&seed_phrase, &encryption_key) {
            Ok((ciphertext, nonce)) => (ciphertext, nonce),
            Err(e) => {
                println!("Encryption failed: {}", e);
                return;
            }
        };

        let encrypted_password = match crypto_service.encrypt(&password, &encryption_key) {
            Ok((ciphertext, nonce)) => (ciphertext, nonce),
            Err(e) => {
                println!("Encryption failed: {}", e);
                return;
            }
        };

        let account_data = json!({
            "account_name": account_name,
            "encrypted_password": encrypted_password.0,
            "password_nonce": hex::encode(encrypted_password.1),
            "encrypted_seed_phrase": encrypted_seed.0,
            "seed_nonce": hex::encode(encrypted_seed.1),
            "encryption_key": hex::encode(encryption_key),
        });

        fs::create_dir_all(STORAGE_DIR).expect("Failed to create storage directory");
        let account_file = format!("{}/{}.json", STORAGE_DIR, account_name);

        if let Err(e) = fs::write(&account_file, account_data.to_string()) {
            println!("Unable to write account data to file: {}", e);
        } else {
            println!("Account '{}' has been created successfully.", account_name);
        }
    }

    pub fn login(account_name: &str) {
        let account_file = format!("{}/{}.json", STORAGE_DIR, account_name);
        let account_data = match fs::read_to_string(&account_file) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to read account data: {}", e);
                return;
            }
        };

        let account_json: Value = match serde_json::from_str(&account_data) {
            Ok(json) => json,
            Err(e) => {
                println!("Failed to parse account data: {}", e);
                return;
            }
        };

        let password = AccountService::get_password("Enter your password: ");

        let encryption_key_str = account_json["encryption_key"].as_str().unwrap();
        let encryption_key_bytes = hex::decode(encryption_key_str).expect("Failed to decode encryption key");
        let encryption_key = Key::<Aes256Gcm>::from_slice(&encryption_key_bytes);

        let encrypted_password = account_json["encrypted_password"].as_str().unwrap();

        let password_nonce_str = account_json["password_nonce"].as_str().unwrap();
        let password_nonce_bytes = hex::decode(password_nonce_str).expect("Failed to decode password nonce");
        let password_nonce = Nonce::<Aes256Gcm>::from_slice(&password_nonce_bytes);

        let decrypted_password = match CryptoService::decrypt(&encrypted_password, &encryption_key, &password_nonce) {
            Ok(decrypted) => decrypted,
            Err(e) => {
                println!("Decryption failed: {}", e);
                return;
            }
        };

        if decrypted_password != password {
            println!("Incorrect password. Please try again.");
            return;
        }

        let state_data = json!({
            "logged_in_account": account_name
        });
        if let Err(e) = fs::write(STATE_FILE, state_data.to_string()) {
            println!("Failed to update login state: {}", e);
        } else {
            println!("Login successful for account '{}'.", account_name);
        }
    }

    pub fn logout() {
        let state_data = match fs::read_to_string(STATE_FILE) {
            Ok(data) => data,
            Err(_) => {
                println!("No user is currently logged in.");
                return;
            }
        };

        let state_json: Value = match serde_json::from_str(&state_data) {
            Ok(json) => json,
            Err(_) => {
                println!("No user is currently logged in.");
                return;
            }
        };

        if state_json["logged_in_account"].is_null() {
            println!("No user is currently logged in.");
            return;
        }

        let state_data = json!({
            "logged_in_account": null
        });
        if let Err(e) = fs::write(STATE_FILE, state_data.to_string()) {
            println!("Failed to update logout state: {}", e);
        } else {
            println!("Logout successful.");
        }
    }

    fn get_password(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut password = String::new();
        io::stdin().read_line(&mut password).expect("Failed to read password");
        password.trim().to_string()
    }
}