# Solify - Automated Test Generator for Solana Programs

solify/
├── common/                    # Shared types and utilities
├── parser/                    # IDL parsing library
├── analyzer_onchain/          # On-chain dependency analysis
├── generator/                 # Test file generation
├── client/                    # Solana RPC client
├── cli/                       # Command-line interface
└── solana-program/            # On-chain Solana program
    └── solify/                # Anchor project

Steps to build:-
1. IDL parser
- example/parse_idl.rs - to test /logs fot parsed idl
    run to parse IDL
```cargo run --example parse_idl -p solify-parser```
2. On chain program
- structure
- instructions
- analyzer modules
- utilities errors
- validation
3. Off chain libraries
- analyzer crate
- generate crate
- client crate
4. CLI & TUI part
- solify init
- solify gen-test
- solify inspect
- solify stats