
/// Run with: cargo run --example parse_idl -p solify-parser

use solify_parser::{parse_idl, get_pda_accounts, get_signer_accounts, get_writable_accounts};

fn main() {
    println!("=== Solify IDL Parser ===\n");
    
    let idl_path = concat!(env!("CARGO_MANIFEST_DIR"), "/idls/dice.json");
    
    println!(" Reading IDL from: {}", idl_path);
    
    let idl_data = match parse_idl(idl_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing IDL: {}", e);
            return;
        }
    };
    
    println!("Successfully parsed IDL!\n");

    println!("Program: {} (v{})", idl_data.name, idl_data.version);
    println!("Instructions: {}", idl_data.instructions.len());
    println!("Accounts: {}", idl_data.accounts.len());
    println!("Types: {}", idl_data.types.len());
    println!();
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Available Instructions:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for (i, instr) in idl_data.instructions.iter().enumerate() {
        println!("  {}. {}", i + 1, instr.name);
    }
    println!();


    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Instruction Details:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for (i, instr) in idl_data.instructions.iter().enumerate() {
        println!(" {}. {}", i+1, instr.name);
        println!("  Accounts: {}", instr.accounts.len());
        for account in &instr.accounts {
            let mut flags = Vec::new();
            if account.is_mut { flags.push("mut"); }
            if account.is_signer { flags.push("signer"); }
            if account.pda.is_some() { flags.push("PDA"); }
            
            let flags_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };
            
            println!("  • {}{}", account.name, flags_str);
            
            if let Some(pda) = &account.pda {
                println!("    PDA Seeds:");
                for seed in &pda.seeds {
                    match seed.kind.as_str() {
                        "const" => {
                            // Display const seeds with their value
                            if seed.value.starts_with("0x") {
                                println!("      - const: {}", seed.value);
                            } else {
                                println!("      - const: \"{}\"", seed.value);
                            }
                        }
                        "account" => println!("      - account: {}", seed.path),
                        "arg" => println!("      - arg: {}", seed.path),
                        _ => println!("      - {}: {}", seed.kind, seed.path),
                    }
                }
                if !pda.program.is_empty() {
                    println!("    Program: {}", pda.program);
                }
            }
        }
        println!("\n  Arguments ({}):", instr.args.len());
        for arg in &instr.args {
            println!("   • {}: {}", arg.name, arg.field_type);
        }
        let pdas = get_pda_accounts(instr);
        let signers = get_signer_accounts(instr);
        let writable = get_writable_accounts(instr);
        
        println!("\n  Analysis:");
        println!("   PDAs: {} {:?}", pdas.len(), pdas);
        println!("   Signers: {} {:?}", signers.len(), signers);
        println!("   Writable: {} {:?}", writable.len(), writable);
    }
    
    if !idl_data.types.is_empty() {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Type Definitions:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for type_def in &idl_data.types {
            println!("  • {} ({})", type_def.name, type_def.kind);
            println!("    Fields: {}", type_def.fields.join(", "));
        }
    }
    
    if !idl_data.errors.is_empty() {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Error Definitions ({}):", idl_data.errors.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for error in &idl_data.errors {
            println!("  • {} (code: {})", error.name, error.code);
            println!("    {}", error.msg);
        }
    }

    if !idl_data.constants.is_empty() {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Constants ({}):", idl_data.constants.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for constant in &idl_data.constants {
            println!("  • {}: {} = {}", constant.name, constant.constant_type, constant.value);
        }
    }

    if !idl_data.events.is_empty() {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Events ({}):", idl_data.events.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for event in &idl_data.events {
            println!("  • {}", event.name);
            if !event.fields.is_empty() {
                println!("    Fields:");
                for field in &event.fields {
                    println!("      - {}: {}", field.name, field.field_type);
                }
            }
        }
    }
}

