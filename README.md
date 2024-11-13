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
- Display native token account balance:
```bash
cargo run -- account balance
```
- Display token account balance:
```bash
cargo run -- account balance-token <token-address>
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
cargo run -- network add --rpc-url <network-url> <network-name> <native-token> <chain-id>
```
- Set url for existing network:
```bash
cargo run -- network set-url <network-name> <network-url>
```
- Get current network information:
```bash
cargo run -- network info
```

### 3. Transaction Management

- Send a native token transaction:
```bash
cargo run -- tx send <amount> <destination-address>
```

- Send a token transaction:
```bash
cargo run -- tx send-token <amount> <destination-address> <token-address>
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
