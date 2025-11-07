# Solify - Automated Test Generator for Solana Programs

- analyzer = all the analyze part will be done under this folder
- common = shared library for structs, errors
- parser = IDL parser library
- client = solan RPC client for handling on chain transactions
- cli = Interface for user
- solana-program = on chain program
- generator = to generate tests

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
