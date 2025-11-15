use anyhow::{ Context, Result };
use std::collections::HashMap;
use std::fs::{ create_dir_all, File };
use std::io::Write;
use std::path::Path;

use solify_common::{
    IdlData,
    SeedComponent,
    SeedType,
    SetupType,
    TestMetadata,
    TestValueType,
};
use solify_common::errors::SolifyError;
use tera::{ Tera, Context as TeraContext };
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AccountInfo {
    original_name: String,
    camel_name: String,
}

pub fn generate_with_tera(
    meta: &TestMetadata,
    idl: &IdlData,
    out_dir: impl AsRef<Path>
) -> Result<()> {
    let out_dir = out_dir.as_ref();
    create_dir_all(out_dir).with_context(|| format!("creating output dir {:?}", out_dir))?;

    let mut tera = Tera::default();
    tera
        .add_raw_template("aggregated_tests.tera", AGGREGATED_TEMPLATE)
        .context("add aggregated template")?;

    let mut ctx = TeraContext::new();

    let program_name = &idl.name;
    let program_name_pascal = cut_program_name(program_name);
    let program_capitalized = capitalize_first_letter(&program_name_pascal);
    let program_name_camel = camel_case(program_name);
    let program_name_pascal_case = to_pascal_case(program_name);
    ctx.insert("program_name", program_name);
    ctx.insert("program_name_pascal", &program_name_pascal);
    ctx.insert("program_capitalized", &program_capitalized);
    ctx.insert("program_name_camel", &program_name_camel);
    ctx.insert("program_name_pascal_case", &program_name_pascal_case);
    

    // setup requirements
    let setup_requirements = meta.setup_requirements.clone();
    let mut map = HashMap::new();
    let mut index = 0;

    for setup_requirement in setup_requirements.iter().cloned() {
        index += 1;

        match setup_requirement.requirement_type {
            SetupType::CreateKeypair => {
                map.insert(index, "Keypair.generate()");
            }
            SetupType::FundAccount => {
                map.insert(index, "FundAccount");
            }
            SetupType::InitializePda => {
                map.insert(index, "PublicKey");
            }
            _ => {
                return Err(SolifyError::InvalidSetupRequirement.into());
            }
        }
    }
    ctx.insert("setup_requirements", &map);

    let mut pda_indices = Vec::new();
    let mut index_1 = 0;

    for r in setup_requirements.iter().cloned() {
        index_1 += 1;

        if r.requirement_type == SetupType::InitializePda {
            pda_indices.push(index_1);
        }
    }

    // pda initialization
    let mut pda_map = HashMap::new();
    let pda_init_sequence = meta.pda_init_sequence.clone();

    for (i, pda_init) in pda_init_sequence.iter().enumerate() {
        if let Some(index) = pda_indices.get(i) {
        let seeds_expr = render_pda_seeds_expression(&pda_init.seeds);
            pda_map.insert(*index, seeds_expr);
        }
    }

    ctx.insert("pda_seeds", &pda_map);

    let mut account_vars: HashMap<String, String> = HashMap::new();

    for ad in meta.account_dependencies.iter() {
        if ad.is_pda {
            if
                let Some((pos, _)) = meta.pda_init_sequence
                    .iter()
                    .enumerate()
                    .find(|(_, p)| p.account_name == ad.account_name)
            {
                let setup_index = pda_indices[pos];
                account_vars.insert(ad.account_name.clone(), format!("pda{}", setup_index));
            } else {
                account_vars.insert(
                    ad.account_name.clone(),
                    format!("/* missing pda for {} */ null", ad.account_name)
                );
            }
        } else if ad.account_name == "authority" {
            account_vars.insert(ad.account_name.clone(), "authorityPubkey".to_string());
        } else if ad.account_name == "system_program" {
            account_vars.insert(ad.account_name.clone(), "SystemProgram.programId".to_string());
        } else {
            account_vars.insert(ad.account_name.clone(), format!("{}", ad.account_name));
        }
    }

    for instruction in &idl.instructions {
        for acc in &instruction.accounts {
            if !account_vars.contains_key(&acc.name) {
                if acc.name == "system_program" || acc.name == "systemProgram" {
                    account_vars.insert(acc.name.clone(), "SystemProgram.programId".to_string());
                } else if acc.name == "authority" {
                    account_vars.insert(acc.name.clone(), "authorityPubkey".to_string());
                } else {
                    if let Some((pos, _)) = meta.pda_init_sequence
                        .iter()
                        .enumerate()
                        .find(|(_, p)| p.account_name == acc.name)
                    {
                        if let Some(setup_index) = pda_indices.get(pos) {
                            account_vars.insert(acc.name.clone(), format!("pda{}", setup_index));
                        }
                    }
                }
            }
        }
    }

    ctx.insert("account_vars", &account_vars);
    let mut instruction_accounts: HashMap<String, Vec<AccountInfo>> = HashMap::new();
    for instruction in &idl.instructions {
        let account_infos: Vec<AccountInfo> = instruction.accounts.iter()
            .map(|acc| {
                AccountInfo {
                    original_name: acc.name.clone(),
                    camel_name: to_camel_case(&acc.name),
                }
            })
            .collect();
        instruction_accounts.insert(instruction.name.clone(), account_infos);
    }
    ctx.insert("instruction_accounts", &instruction_accounts);

    let mut processed_test_cases = meta.test_cases.clone();
    for test_case in &mut processed_test_cases {
        for arg_value in &mut test_case.positive_cases {
            for arg in &mut arg_value.argument_values {
                arg.value_type = convert_to_typescript_value(arg.value_type.clone());
            }
        }
        for arg_value in &mut test_case.negative_cases {
            for arg in &mut arg_value.argument_values {
                arg.value_type = convert_to_typescript_value(arg.value_type.clone());
            }
        }
    }
    ctx.insert("instruction_tests", &processed_test_cases);

    let rendered = tera.render("aggregated_tests.tera", &ctx).context("render tera")?;

    let out_path = out_dir.join(format!("{}.ts", program_name_pascal));
    let mut f = File::create(&out_path).with_context(|| format!("create file {:?}", out_path))?;
    f.write_all(rendered.as_bytes()).with_context(|| format!("write file {:?}", out_path))?;

    println!("Wrote {}", out_path.display());
    Ok(())
}

const AGGREGATED_TEMPLATE: &str =
    r#"
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { {{ program_name_pascal_case }} } from "../target/types/{{ program_name }}";
import { assert } from "chai";
import { Keypair, SystemProgram, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("{{ program_name | default(value='program') }}", () => {
    // Configure the client
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;

    const program = anchor.workspace.{{ program_name }} as Program<{{ program_name_pascal_case }}>;

    // Setup Requirements
    // keypair decelarations
    {%- set keypair_found = false %}
    {%- for id, code in setup_requirements %}
    {%- if code == "Keypair.generate()" %}
    {%- if not keypair_found %}
    {%- set keypair_found = true %}
    const authority = Keypair.generate();
    const authorityPubkey = authority.publicKey;
    {%- else %}
    const user{{ id }} = Keypair.generate();
    const user{{ id }}Pubkey = user{{ id }}.publicKey;
    {%- endif %}
    {%- endif %}
    {%- endfor %}

    // PDA Decelaration
    {%- for id, code in setup_requirements %}
    {%- if code == "PublicKey" %}
    let pda{{ id }}: PublicKey;
    let bump{{ id }}: number;
    {%- endif %}
    {%- endfor %}

    before(async () => {
        // ----- Airdrop for each user Keypair -----
        {%- set keypair_found_airdrop = false %}
        {%- for id, code in setup_requirements %}
        {%- if code == "Keypair.generate()" %}
        {%- if not keypair_found_airdrop %}
        {%- set keypair_found_airdrop = true %}
        const sig{{ id }} = await connection.requestAirdrop(authorityPubkey, 10 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig{{ id }}, "confirmed");
        {%- else %}
        const sig{{ id }} = await connection.requestAirdrop(user{{ id }}Pubkey, 10 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig{{ id }}, "confirmed");
        {%- endif %}
        {%- endif %}
        {%- endfor %}

        // ----- PDA Initialization -----
        {%- for id, seeds in pda_seeds %}
        [pda{{ id }}, bump{{ id }}] = PublicKey.findProgramAddressSync(
            {{ seeds }},
            program.programId
        );
        {%- endfor %}

    });

    {%- macro render_accounts(account_list) -%}
    {%- for acc in account_list %}
    {%- set var = account_vars[acc] | default(value='/* missing */ null') %}
    {{ acc }}: {{ var }}{%- if not loop.last %},{%- endif %}
    {%- endfor %}
    {%- endmacro %}

    {# ---------------- INSTRUCTION DESCRIBE BLOCKS ---------------- #}

    {%- for instr in instruction_tests %}


    {# ---------- POSITIVE TESTS ---------- #}
    {%- for test in instr.positive_cases %}
    it("{{ test.description }}", async () => {
        // Prepare arguments
        {%- for arg in test.argument_values %}
        {%- if arg.value_type.variant == "Valid" %}
        const {{ arg.argument_name }}Value = {{ arg.value_type.description }};
        {%- elif arg.value_type.variant == "Invalid" %}
        const {{ arg.argument_name }}Value = {{ arg.value_type.description }};
        {%- else %}
        const {{ arg.argument_name }}Value = null;
        {%- endif %}
        {%- endfor %}
        // Execute instruction
        try {
            await program.methods
                .{{ instr.instruction_name }}(
                    {%- for arg in test.argument_values %}
                    {{ arg.argument_name }}Value{%- if not loop.last %},{%- endif %}
                    {%- endfor %}
                )
                .accountsStrict({
                    {%- if instruction_accounts[instr.instruction_name] %}
                    {%- for acc_info in instruction_accounts[instr.instruction_name] %}
                    {%- set js_var = account_vars[acc_info.original_name] | default(value="null") %}
                    {{ acc_info.camel_name }}: {{ js_var }}{%- if not loop.last %},{%- endif %}
                    {%- endfor %}
                    {%- endif %}
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    {%- endfor %}
    {# ---------- NEGATIVE TESTS ---------- #}
    {%- for test in instr.negative_cases %}
    it("{{ test.description }}", async () => {
        // Prepare arguments
        {%- for arg in test.argument_values %}
        {%- if arg.value_type.variant == "Valid" %}
        const {{ arg.argument_name }}Value = {{ arg.value_type.description }};
        {%- elif arg.value_type.variant == "Invalid" %}
        const {{ arg.argument_name }}Value = {{ arg.value_type.description }};
        {%- else %}
        const {{ arg.argument_name }}Value = null;
        {%- endif %}
        {%- endfor %}
        // Execute instruction expecting failure
        try {
            await program.methods
                .{{ instr.instruction_name }}(
                    {%- for arg in test.argument_values %}
                    {{ arg.argument_name }}Value{%- if not loop.last %},{%- endif %}
                    {%- endfor %}
                )
                .accountsStrict({
                    {%- if instruction_accounts[instr.instruction_name] %}
                    {%- for acc_info in instruction_accounts[instr.instruction_name] %}
                    {%- set js_var = account_vars[acc_info.original_name] | default(value="null") %}
                    {{ acc_info.camel_name }}: {{ js_var }}{%- if not loop.last %},{%- endif %}
                    {%- endfor %}
                    {%- endif %}
                })
                .signers([
                    authority
                ])
                .rpc();
        } catch (err) {
            {%- if test.expected_outcome.variant == "Failure" %}
            assert(err.message.includes("{{ test.expected_outcome.error_message }}"));
            {%- endif %}
        }
    });
    {%- endfor %}

    {%- endfor %}

})

"#;

// ------------------- Helper functions (rendering helpers) -------------------

fn render_pda_seeds_expression(seeds: &[SeedComponent]) -> String {
    let parts: Vec<String> = seeds
        .iter()
        .map(|seed| {
            match seed.seed_type {
                SeedType::Static => { format!("Buffer.from(\"{}\")", seed.value) }
                SeedType::AccountKey => { format!("{}Pubkey.toBuffer()", seed.value) }
                SeedType::Argument => { format!("Buffer.from(String({}))", seed.value) }
            }
        })
        .collect();

    format!("[{}]", parts.join(", "))
}

fn cut_program_name(s: &str) -> String {
    s.split('_').next().unwrap_or(s).to_string()
}

fn capitalize_first_letter(s: &str) -> String {
    s.chars().next().unwrap_or('A').to_uppercase().to_string() + &s[1..]
}

fn camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    if parts.is_empty() {
        return String::new();
    }
    let first = parts[0].to_lowercase();
    let rest: String = parts[1..].iter()
        .map(|word| {
            if word.is_empty() {
                String::new()
    } else {
                word.chars().next().unwrap().to_uppercase().to_string() + &word[1..].to_lowercase()
            }
        })
        .collect();
    first + &rest
}

fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    if parts.is_empty() {
        return String::new();
    }
    let first = parts[0].to_lowercase();
    let rest: String = parts[1..].iter()
        .map(|word| {
            if word.is_empty() {
                String::new()
            } else {
                let mut chars = word.chars();
                if let Some(first_char) = chars.next() {
                    first_char.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                } else {
                    String::new()
                }
            }
        })
        .collect();
    first + &rest
}

fn to_pascal_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').collect();
    if parts.is_empty() {
        return String::new();
    }
    parts.iter()
        .map(|word| {
            if word.is_empty() {
                String::new()
            } else {
                let mut chars = word.chars();
                if let Some(first_char) = chars.next() {
                    first_char.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                } else {
                    String::new()
                }
            }
        })
        .collect()
}

fn convert_to_typescript_value(value_type: TestValueType) -> TestValueType {
    match value_type {
        TestValueType::Valid { description } => {
            TestValueType::Valid {
                description: convert_rust_to_typescript(&description),
            }
        }
        TestValueType::Invalid { description, reason } => {
            TestValueType::Invalid {
                description: convert_rust_to_typescript(&description),
                reason,
            }
        }
    }
}

fn convert_rust_to_typescript(value: &str) -> String {
    let trimmed = value.trim();
    
    match trimmed {
        "u64::MAX" => "new anchor.BN(\"18446744073709551615\")".to_string(),
        "u64::MIN" => "new anchor.BN(\"0\")".to_string(),
        "u32::MAX" => "new anchor.BN(\"4294967295\")".to_string(),
        "u32::MIN" => "new anchor.BN(\"0\")".to_string(),
        "u16::MAX" => "new anchor.BN(\"65535\")".to_string(),
        "u16::MIN" => "new anchor.BN(\"0\")".to_string(),
        "u8::MAX" => "new anchor.BN(\"255\")".to_string(),
        "u8::MIN" => "new anchor.BN(\"0\")".to_string(),
        "i64::MAX" => "new anchor.BN(\"9223372036854775807\")".to_string(),
        "i64::MIN" => "new anchor.BN(\"-9223372036854775808\")".to_string(),
        "i32::MAX" => "new anchor.BN(\"2147483647\")".to_string(),
        "i32::MIN" => "new anchor.BN(\"-2147483648\")".to_string(),
        "i16::MAX" => "new anchor.BN(\"32767\")".to_string(),
        "i16::MIN" => "new anchor.BN(\"-32768\")".to_string(),
        "i8::MAX" => "new anchor.BN(\"127\")".to_string(),
        "i8::MIN" => "new anchor.BN(\"-128\")".to_string(),
        _ => {
            if let Ok(_) = trimmed.parse::<i128>() {
                format!("new anchor.BN(\"{}\")", trimmed)
            } else if let Ok(_) = trimmed.parse::<f64>() {
                if trimmed.contains('.') {
                    trimmed.to_string()
                } else {
                    format!("new anchor.BN(\"{}\")", trimmed)
                }
            } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
                trimmed.to_string()
            } else if trimmed == "true" || trimmed == "false" {
                trimmed.to_string()
            } else if trimmed.starts_with("new ") || trimmed.starts_with("authority.") || trimmed.contains("Pubkey") {
                trimmed.to_string()
            } else {
                if trimmed.starts_with('"') {
                    trimmed.to_string()
                } else {
                    format!("\"{}\"", trimmed)
                }
            }
        }
    }
}