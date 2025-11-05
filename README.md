# Solify - Automated Test Generator for Solana Programs

- analyzer = all the analyze part will be done under this folder
- common = shared library for structs, errors
- parser = IDL parser library
- client = solan RPC client for handling on chain transactions
- cli = Interface for user
- solana-program = on chain program
- generator = to generate tests

Steps to build:-
1. On chain program
- structure
- instructions
- analyzer modules
- utilities errors
- validation
2. Off chain libraries
- parser create
- analyzer crate
- generate crate
- client crate
3. CLI & TUI part
- solify init
- solify gen-test
- solify inspect
- solify stats