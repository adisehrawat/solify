use anyhow::{Context, Result};
use solify_common::{
    IdlData, IdlInstruction, IdlAccountItem, IdlField, IdlAccount, IdlTypeDef, 
    IdlPda, IdlSeed, IdlError, IdlConstant, IdlEvent, ParsedIdl
};

use solana_sdk::pubkey::Pubkey;
use std::fs;
use std::path::Path;


pub fn parse_idl<P: AsRef<Path>>(idl_path: P) -> Result<IdlData> {
    let path = idl_path.as_ref();
    let idl_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read IDL file at {:?}", path))?;
    let parsed_idl: ParsedIdl = serde_json::from_str(&idl_content)
        .with_context(|| {
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&idl_content) {
                format!("Invalid JSON: {}", e)
            } else {
                "Failed to deserialize IDL JSON - structure mismatch".to_string()
            }
        })?;
    
    convert_to_idl_data(parsed_idl)
}

fn convert_to_idl_data(parsed: ParsedIdl) -> Result<IdlData> {
    if parsed.instructions.is_empty() {
        anyhow::bail!("IDL must have at least one instruction");
    }
    
    Ok(IdlData {
        name: parsed.metadata.name,
        version: parsed.metadata.version,
        instructions: parsed.instructions.into_iter().map(convert_instruction).collect(),
        accounts: parsed.accounts.into_iter().map(convert_account).collect(),
        types: parsed.types.into_iter().map(convert_type).collect(),
        errors: parsed.errors.into_iter().map(convert_error).collect(),
        constants: parsed.constants.into_iter().map(convert_constant).collect(),
        events: parsed.events.into_iter().map(convert_event).collect(),
    })
}

fn convert_error(error: solify_common::ErrorDef) -> IdlError {
    IdlError {
        code: error.code,
        name: error.name,
        msg: error.msg,
    }
}

fn convert_constant(constant: solify_common::ConstantDef) -> IdlConstant {
    IdlConstant {
        name: constant.name,
        constant_type: constant.constant_type,
        value: constant.value,
    }
}

fn convert_event(event: solify_common::EventDef) -> IdlEvent {
    IdlEvent {
        name: event.name,
        discriminator: event.discriminator,
        fields: event.fields.into_iter().map(convert_field_def).collect(),
    }
}

fn convert_field_def(field: solify_common::FieldDef) -> IdlField {
    IdlField {
        name: field.name,
        field_type: type_to_string(&field.field_type),
    }
}

fn convert_instruction(instr: solify_common::Instruction) -> IdlInstruction {
    IdlInstruction {
        name: instr.name,
        accounts: instr.accounts.into_iter().map(convert_account_info).collect(),
        args: instr.args.into_iter().map(convert_argument).collect(),
        docs: instr.docs,
    }
}

fn convert_account_info(acc: solify_common::AccountInfo) -> IdlAccountItem {
    IdlAccountItem {
        name: acc.name,
        is_mut: acc.writable,
        is_signer: acc.signer,
        is_optional: acc.optional,
        docs: acc.docs,
        pda: acc.pda.map(convert_pda_config),
    }
}

fn convert_pda_config(pda: solify_common::PdaConfig) -> IdlPda {
    let program = pda.program
        .and_then(|prog| {
            if prog.kind == "const" {
                prog.value.and_then(|bytes| bytes_to_pubkey(&bytes))
            } else {
                None
            }
        })
        .unwrap_or_default();
    
    IdlPda {
        seeds: pda.seeds.into_iter().map(convert_pda_seed).collect(),
        program,
    }
}

fn convert_pda_seed(seed: solify_common::PdaSeed) -> IdlSeed {
    let value = if seed.kind == "const" {
        seed.value
            .as_ref()
            .map(|bytes| {
                if bytes.len() == 32 {
                    if let Some(pubkey_str) = bytes_to_pubkey(bytes) {
                        return pubkey_str;
                    }
                }
                
                if is_likely_utf8_string(bytes) {
                    String::from_utf8(bytes.clone())
                        .unwrap_or_else(|_| bytes_to_hex(bytes))
                } else {
                    bytes_to_hex(bytes)
                }
            })
            .unwrap_or_default()
    } else {
        String::new()
    };
    
    IdlSeed {
        kind: seed.kind,
        path: seed.path,
        value,
    }
}

fn is_likely_utf8_string(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    
    if let Ok(s) = std::str::from_utf8(bytes) {
        let printable_count = s.chars().filter(|c| {
            c.is_alphanumeric() || c.is_whitespace() || "_-./".contains(*c)
        }).count();
        
        let ratio = printable_count as f32 / s.chars().count() as f32;
        ratio > 0.8
    } else {
        false
    }
}

fn bytes_to_pubkey(bytes: &[u8]) -> Option<String> {
    if bytes.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        let pubkey = Pubkey::new_from_array(arr);
        Some(pubkey.to_string())
    } else {
        None
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.len() <= 8 {
        format!("0x{}", bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>())
    } else {
        let preview: String = bytes.iter().take(4).map(|b| format!("{:02x}", b)).collect();
        format!("0x{}... ({} bytes)", preview, bytes.len())
    }
}

fn convert_argument(arg: solify_common::ArgumentDef) -> IdlField {
    IdlField {
        name: arg.name,
        field_type: type_to_string(&arg.arg_type),
    }
}

fn convert_account(acc: solify_common::AccountDef) -> IdlAccount {
    IdlAccount {
        name: acc.name,
        fields: vec![],
    }
}

fn convert_type(type_def: solify_common::TypeDef) -> IdlTypeDef {
    match type_def.type_kind {
        solify_common::TypeKind::Struct { fields } => {
            IdlTypeDef {
                name: type_def.name,
                kind: "struct".to_string(),
                fields: fields.into_iter().map(|f| f.name).collect(),
            }
        }
        solify_common::TypeKind::Enum { variants } => {
            IdlTypeDef {
                name: type_def.name,
                kind: "enum".to_string(),
                fields: variants.into_iter().map(|v| v.name).collect(),
            }
        }
    }
}

fn type_to_string(idl_type: &solify_common::IdlType) -> String {
    match idl_type {
        solify_common::IdlType::Simple(s) => s.clone(),
        solify_common::IdlType::Vec { vec } => {
            format!("Vec<{}>", type_to_string(vec))
        }
        solify_common::IdlType::Option { option } => {
            format!("Option<{}>", type_to_string(option))
        }
        solify_common::IdlType::Array { array } => {
            let (inner, size) = array;
            format!("[{}; {}]", type_to_string(inner), size)
        }
        solify_common::IdlType::Defined { defined } => {
            match defined {
                solify_common::DefinedType::Simple(name) => name.clone(),
                solify_common::DefinedType::Generic { name, generics } => {
                    if generics.is_empty() {
                        name.clone()
                    } else {
                        let generic_strs: Vec<String> = generics.iter().map(type_to_string).collect();
                        format!("{}<{}>", name, generic_strs.join(", "))
                    }
                }
            }
        }
    }
}


pub fn get_instruction_names<P: AsRef<Path>>(idl_path: P) -> Result<Vec<String>> {
    let idl_data = parse_idl(idl_path)?;
    Ok(idl_data.instructions.iter().map(|i| i.name.clone()).collect())
}

pub fn find_instruction<'a>(idl_data: &'a IdlData, name: &str) -> Option<&'a IdlInstruction> {
    idl_data.instructions.iter().find(|i| i.name == name)
}

pub fn get_pda_accounts(instruction: &IdlInstruction) -> Vec<String> {
    instruction
        .accounts
        .iter()
        .filter(|acc| acc.pda.is_some())
        .map(|acc| acc.name.clone())
        .collect()
}

pub fn get_signer_accounts(instruction: &IdlInstruction) -> Vec<String> {
    instruction
        .accounts
        .iter()
        .filter(|acc| acc.is_signer)
        .map(|acc| acc.name.clone())
        .collect()
}

pub fn get_writable_accounts(instruction: &IdlInstruction) -> Vec<String> {
    instruction
        .accounts
        .iter()
        .filter(|acc| acc.is_mut)
        .map(|acc| acc.name.clone())
        .collect()
}

pub fn get_program_id<P: AsRef<Path>>(idl_path: P) -> Result<String> {
    let path = idl_path.as_ref();
    let idl_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read IDL file at {:?}", path))?;
    let parsed_idl: ParsedIdl = serde_json::from_str(&idl_content)
        .with_context(|| {
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&idl_content) {
                format!("Invalid JSON: {}", e)
            } else {
                "Failed to deserialize IDL JSON - structure mismatch".to_string()
            }
        })?;
    let program_id = parsed_idl.address;
    Ok(program_id)
}