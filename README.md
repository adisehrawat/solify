# Solify

[![crates.io](https://img.shields.io/crates/v/solify.svg)](https://crates.io/crates/solify)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**Solify** is a powerful CLI tool that automatically generates comprehensive test suites for Solana Anchor programs. By analyzing your program's IDL (Interface Definition Language) file, Solify creates TypeScript test files with positive and negative test cases, handles account setup, PDA initialization, and provides an interactive interface for test generation.

## Features

-  **Automated Test Generation**: Generate complete test suites from IDL files
-  **Smart Analysis**: Analyzes program dependencies, account ordering, and PDA requirements
-  **Comprehensive Coverage**: Creates both positive and negative test cases
-  **Interactive TUI**: Beautiful terminal user interface for guided test generation
-  **Transaction Inspector**: Inspect and analyze Solana transactions
-  **Anchor Integration**: Automatically detects and integrates with Anchor project structures
-  **Account Management**: Handles keypair generation, airdrops, and PDA initialization
-  **On-chain Processing**: Executes instructions on-chain to gather real transaction data

## Installation

### From crates.io (Recommended)

```bash
cargo install solify
```

### From Source

```bash
git clone https://github.com/adisehrawat/solify.git
cd solify
cargo build --release
cargo install --path cli
```

### Requirements

- Rust 1.70+ (for building from source)
- Solana CLI tools (for transaction inspection)
- Node.js and Anchor (for running generated tests)

## Quick Start

### Generate Tests for Your Anchor Program

1. **Navigate to your Anchor project**:
   ```bash
   cd your-anchor-project
   ```

2. **Build your program** to generate the IDL:
   ```bash
   anchor build
   ```

3. **Run Solify** to generate tests:
   ```bash
   solify gen-test
   ```

   Or specify custom paths:
   ```bash
   solify gen-test --idl target/idl/your_program.json --output tests
   ```

4. **Follow the interactive prompts**:
   - Select the order of instructions to test
   - Provide your wallet keypair path (default: `~/.config/solana/id.json`)
   - Enter a paraphrase for test metadata
   - Watch as Solify analyzes your program and generates tests

5. **Run the generated tests**:
   ```bash
   anchor test
   ```

### Inspect a Transaction

```bash
solify inspect <transaction-signature>
```

Example:
```bash
solify inspect 5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmGpnD9i8X5zgD3A6i8j5Y3vJ8vK8vK8vK8
```


## Usage Guide

### Command: `gen-test`

Generates comprehensive test suites for your Solana Anchor program.

**Syntax:**
```bash
solify gen-test [OPTIONS]
```

**Options:**
- `-i, --idl <PATH>`: Path to IDL file or directory containing IDL files (default: `target/idl`)
- `-o, --output <PATH>`: Output directory for generated test files (default: `tests`)
- `--rpc-url <URL>`: Solana RPC endpoint URL (default: `https://api.devnet.solana.com`)
- `-v, --verbose`: Enable verbose logging

**Examples:**

```bash
# Use default paths (target/idl -> tests)
solify gen-test

# Specify custom IDL file
solify gen-test --idl target/idl/my_program.json

# Specify custom output directory
solify gen-test --output my-tests

# Use mainnet RPC (for production programs)
solify gen-test --rpc-url https://api.mainnet-beta.solana.com

# Enable verbose output
solify gen-test --verbose
```

**Interactive Flow:**

1. **IDL Selection**: Solify will automatically find IDL files in the specified directory
2. **Instruction Ordering**: Select the order in which instructions should be tested
3. **Wallet Configuration**: Provide the path to your wallet keypair
4. **Metadata**: Enter a paraphrase/description for the test metadata
5. **Analysis**: Solify analyzes your program structure, dependencies, and requirements
6. **On-chain Execution**: Optionally executes instructions on-chain to gather real data
7. **Test Generation**: Generates TypeScript test files with comprehensive test cases

**Generated Test Structure:**

The generated tests include:
- Setup code with keypair generation and airdrops
- PDA initialization for all required PDAs
- Positive test cases for each instruction
- Negative test cases with error handling
- Proper account management and signers
- TypeScript/Anchor test framework integration

### Command: `inspect`

Inspect and analyze Solana transactions with a beautiful TUI interface.

**Syntax:**
```bash
solify inspect <SIGNATURE> [OPTIONS]
```

**Arguments:**
- `SIGNATURE`: Transaction signature to inspect


**Examples:**

```bash
# Inspect a transaction on devnet
solify inspect 5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmGpnD9i8X5zgD3A6i8j5Y3vJ8vK8vK8vK8


**Inspection Features:**

- Transaction status and confirmation details
- Instruction breakdown with parsed data
- Account information and balances
- Program logs and execution traces
- Compute units consumed
- Fee information
- Return data (if available)
```

## Project Structure

```
solify/
├── cli/              # CLI application and user interface
├── parser/           # IDL parsing and analysis
├── analyzer/         # Program analysis (dependencies, PDAs, account ordering)
├── generator/        # Test file generation using templates
├── client/           # Solana RPC client for on-chain operations
├── common/           # Shared types, errors, and utilities
└── solana-program/   # On-chain Solana program (if applicable)
```

### Key Components

- **CLI**: Command-line interface with interactive TUI
- **Parser**: Parses Anchor IDL files and extracts program structure
- **Analyzer**: Analyzes program dependencies, detects PDAs, determines account ordering
- **Generator**: Generates TypeScript test files using template engine
- **Client**: Handles Solana RPC interactions for on-chain data gathering

## How It Works

1. **IDL Parsing**: Reads and parses your Anchor program's IDL file
2. **Dependency Analysis**: Analyzes instruction dependencies and execution order
3. **Account Analysis**: Detects PDAs, determines account requirements, and ordering
4. **Test Case Generation**: Creates positive and negative test cases for each instruction
5. **On-chain Execution** (optional): Executes instructions on-chain to gather real transaction data
6. **Template Rendering**: Generates TypeScript test files using Handlebars templates
7. **Integration**: Outputs tests compatible with Anchor's test framework

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/adisehrawat/solify.git
cd solify

# Build the project
cargo build --release

# Run tests
cargo test

# Build the CLI
cargo build --release -p solify
```

### Running Examples

```bash
# Parse an IDL file
cargo run --example parse_idl -p solify-parser
```

### Project Dependencies

The project uses a Rust workspace with the following crates:
- `solify-common`: Shared types and error definitions
- `solify-parser`: IDL parsing functionality
- `solify-analyzer`: Program analysis logic
- `solify-generator`: Test generation engine
- `solify-client`: Solana RPC client wrapper

## Configuration


### Wallet Configuration

By default, Solify looks for your wallet at `~/.config/solana/id.json`. You can specify a different path during the interactive flow or by modifying the default in the code.


## Troubleshooting

### Common Issues

**Issue**: "IDL file not found"
- **Solution**: Ensure you've built your Anchor program with `anchor build`, or specify the correct path with `--idl`

**Issue**: "Failed to connect to RPC"
- **Solution**: Check your internet connection and RPC endpoint. Try a different RPC URL with `--rpc-url`

**Issue**: "Wallet not found"
- **Solution**: Ensure your Solana wallet exists at `~/.config/solana/id.json` or provide the correct path during the interactive flow

**Issue**: "Insufficient funds"
- **Solution**: Airdrop SOL to your wallet: `solana airdrop 2` (on devnet)

**Issue**: "Generated tests fail"
- **Solution**: Review the generated test file and adjust account setup or instruction parameters as needed

### Getting Help

- Check the [Issues](https://github.com/adisehrawat/solify/issues) page
- Review the generated test files for debugging
- Enable verbose mode: `solify gen-test --verbose`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built for the Solana and Anchor ecosystems
- Inspired by the need for better testing tools in the Solana development workflow

## Author

**Aditya Sehrawat**
- Email: sehrawataditya22@gmail.com
- GitHub: [@adisehrawat](https://github.com/adisehrawat)

---

**Made with pure Rustling mind for the Solana community**
