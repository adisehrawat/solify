# Solify - Automated Test Generator for Solana Programs

solify/                                    # Root workspace directory
├── Cargo.toml                             # Workspace configuration (defines all crates)
├── .gitignore                             # Git ignore patterns
├── README.md                              # Project documentation
│
├── common/                                # Shared Types & Utilities
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                         # Module exports
│       ├── types.rs                       # Shared data structures (IDL, TestMetadata, etc.)
│       └── error.rs                       # Custom error types
│
├── parser/                                # IDL Parser Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                         # Parser entry point
│       └── idl_parser.rs                  # IDL parsing logic
│
├── analyzer_onchain/                              #  Dependency Analysis Library for on chain testing prupose only
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                         # Analyzer entry point
│       ├── dependency_graph.rs            # Account dependency graph logic
│       ├── pda_detector.rs                # PDA detection utilities
│       ├── test_case_generator.rs         # Test-case generation logic
│       ├── account_order.rs               # Account order logic
│       ├── setup_generator.rs             # Setup Generator logic
│
├── generator/                             # Test File Generator Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                         # Generator entry point
│
├── client/                                # Solana RPC Client Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│
├── cli/                                   # Command Line Interface
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # CLI entry point
│
└── solana-program/                        # On-chain Solana Program
    └── solify/                            # Anchor project root
        │
        ├── programs/solify/               # Actual Solana program
        │   └── src/
        │       ├── lib.rs                 # Program entry + instructions
        │       │
        │       ├── instructions/          # Instruction handlers
        │       │   ├── mod.rs
        │       │   ├── initialize_user.rs
        │       │   └── generate_metadata.rs
        │       │
        │       ├── states/                # On-chain accounts
        │       │   ├── mod.rs
        │       │   └── user_config.rs
        │       │   └── program_history.rs
        │       │
        │       ├── analyzer/              # On-chain analysis logic
        │       │   ├── mod.rs
        │       │   ├── dependency_analyzer.rs
        │       │   ├── pda_analyzer.rs
        │       │   └── account_order.rs
        │       │   └── setup_generator.rs
        │       │   └── test_case_generator.rs
        │       │
        │       ├── types/                 # Helper utilities
        │       │   ├── mod.rs
        │       │   └── idl_data.rs
        │       │   └── dependencies.rs
        │       │   └── test_metadata.rs
        │       │
        │       └── errors.rs              # Program-specific errors
        │       └── constants.rs              # Constants



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