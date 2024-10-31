# CLI Crypto Wallet

CLI wallet for EVM compatible blockchains

## About

## Usage

### Installation

### 1. Account Management 

- Create a new account:
```bash
cargo run -- account create <account-name>
```
- Login to an existing account:
```bash
cargo run -- account login <account-name>
```
- List all accounts:
```bash
cargo run -- account list
```
- Logout of the current account:
```bash
cargo run -- account logout
```
- Display account balance:
```bash
cargo run -- account balance
```
- Display account information:
```bash
cargo run -- account info
```

### 2. Network Management

- Switch network:
```bash
cargo run -- network switch <network-name>
```
- List all networks:
```bash
cargo run -- network list
```
- Add a new network:
```bash
cargo run -- network add <network-name> <network-url>
```

### 3. Transaction Management

- Send a transaction:
```bash
cargo run -- tx send <amount> <destination-address>
```

Optionally gas price and gas limit can be specified:
```bash
cargo run -- tx send <amount> <destination-address> --gas-price <gas-price> --gas-limit <gas-limit>
```

- View transaction history:
```bash
cargo run -- tx history
```

- Get transaction details:
```bash
cargo run -- tx info <tx-hash>
```
