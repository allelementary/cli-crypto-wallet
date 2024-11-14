# VAULTY: CLI Crypto Wallet

CLI wallet for EVM compatible blockchains

<br/>
<p align="center">
<img src="./vaulty-logo.webp" width="400" alt="vaulty-logo">
</p>
<br/>

## About

Vaulty is a command-line (CLI) crypto wallet application designed for EVM-compatible blockchains. It provides a secure, 
streamlined interface for managing multiple accounts and interacting with various networks. Key features include:

- Wallet Management: Create and manage multiple wallets, supporting separate accounts within the CLI.
- Transaction Management: Send transactions, retrieve account balances, view transaction history, 
and add custom networks.
- Network Flexibility: Add custom networks and specify provider URLs to connect with different EVM-compatible chains.
- Security: All sensitive information is securely stored using AES-256 encryption, ensuring your data remains safe.

Vaulty simplifies crypto wallet operations, all within the command line.

## Features
- **Multiple Accounts**: Manage multiple wallet accounts within the same CLI instance.
- **Secure Storage**: Protects sensitive information with AES-256 encryption.
- **Transaction Support**: Send native and token transactions, with customizable gas settings.
- **Network Flexibility**: Add, switch, and configure custom networks.
- **Account Information**: Retrieve balances, transaction history, and account details.

## Usage

### Installation

##### Prerequisites
- Rust: Make sure you have Rust installed. You can install it via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

##### Install Vaulty
**Option 1: Install via Crates.io**

Vaulty is published on Crates.io, you can install it with:
```bash
cargo install vaulty
```

**Option 2: Install from GitHub**

Alternatively, you can install directly from the GitHub repository:
```bash
cargo install --git https://github.com/allelementary/cli-crypto-wallet
```

##### Post-Installation

Once installed, ensure the binary is accessible from your system’s PATH. You can verify the installation by running:
```bash
vaulty --help
```

If vaulty is not found, add Cargo’s bin directory to your PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
Add this line to your shell profile (e.g., ~/.bashrc or ~/.zshrc) to make it permanent.

#### Default Networks
Vaulty includes support for several popular EVM-compatible networks by default. These networks are preconfigured 
with names, chain IDs, and native token symbols, but they do not come with pre-set RPC URLs. You’ll need to add 
RPC URLs manually to connect Vaulty to these networks.

The default networks are:

- **Ethereum**
    - Chain ID: 1
    - Native Token: ETH

- **Polygon**
    - Chain ID: 137
    - Native Token: POL

- **Optimism**
    - Chain ID: 10
    - Native Token: ETH

- **Binance Smart Chain (BSC)**
    - Chain ID: 56
    - Native Token: BNB

- **Arbitrum**
    - Chain ID: 42161
    - Native Token: ETH

#### Testnet Networks

- **Ethereum Sepolia**
    - Chain ID: 11155111
    - Native Token: ETH

- **Polygon Amoy**
    - Chain ID: 80002
    - Native Token: POL

- **Optimism Sepolia**
    - Chain ID: 11155420
    - Native Token: ETH

- **Binance Smart Chain Testnet (BSC)**
    - Chain ID: 97
    - Native Token: BNB

- **Arbitrum Sepolia**
    - Chain ID: 421614
    - Native Token: ETH


#### Adding RPC URLs
To connect to any of these networks, you’ll need to provide an RPC URL. You can find reliable RPC URLs for each 
network on [Chainlist](https://chainlist.org/), a community-curated resource for network configuration information.

To add an RPC URL for a network in Vaulty:
```bash
vaulty network set-url <network-name> <rpc-url>
```

Once configured, you can easily switch between networks, retrieve balances, and send transactions.


### 1. Account Management 

- Create a new account:
```bash
vaulty account create <account-name>
```
- Login to an existing account:
```bash
vaulty account login <account-name>
```
- List all accounts:
```bash
vaulty account list
```
- Logout of the current account:
```bash
vaulty account logout
```
- Display native token account balance:
```bash
vaulty account balance
```
- Display token account balance:
```bash
vaulty account balance-token <token-address>
```
- Display account information:
```bash
vaulty account info
```

### 2. Network Management

- Switch network:
```bash
vaulty network switch <network-name>
```
- List all networks:
```bash
vaulty network list
```
- Add a new network:
```bash
vaulty network add --rpc-url <network-url> <network-name> <native-token> <chain-id>
```
- Set url for existing network:
```bash
vaulty network set-url <network-name> <network-url>
```
- Get current network information:
```bash
vaulty network info
```

### 3. Transaction Management

- Send a native token transaction:
```bash
vaulty tx send <amount> <destination-address>
```

- Send a token transaction:
```bash
vaulty tx send-token <amount> <destination-address> <token-address>
```

Optionally gas price and gas limit can be specified:
```bash
vaulty tx send <amount> <destination-address> --gas-price <gas-price> --gas-limit <gas-limit>
```

- View transaction history:
```bash
vaulty tx history
```

- Get transaction details:
```bash
vaulty tx info <tx-hash>
```

## Contributing
Contributions are welcome! To report bugs or suggest new features, please open an issue on the GitHub repository.
For pull requests:
- Fork the repository and create a new branch.
- Commit your changes with clear, descriptive messages.
- Submit a pull request, and we’ll review it promptly.

## Contact
For questions or support, please reach out via the GitHub repository 
[issues](https://github.com/allelementary/cli-crypto-wallet/issues).

## License
Vaulty is open-source software licensed under the [MIT License](https://opensource.org/licenses/MIT).

