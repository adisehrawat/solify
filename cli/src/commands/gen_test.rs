use anyhow::{ Context, Result };
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use log::info;
use ratatui::layout::{ Constraint, Direction, Layout };
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solify_client::SolifyClient;
use solify_common::TestMetadata;
use solify_parser::{ get_program_id, parse_idl };
use std::str::FromStr;
use std::{ fs, path::PathBuf };
use std::time::Duration;
use solana_commitment_config::CommitmentConfig;
use solify_generator::generate_with_tera;
use solify_analyzer::DependencyAnalyzer;

use crate::tui::{
    AppEvent,
    EventHandler,
    init_terminal,
    render_banner,
    render_info_box,
    render_key_hints,
    render_list,
    render_progress,
    render_status,
    restore_terminal,
};

enum AppState {
    #[allow(dead_code)]
    SelectingInstructions,
    Analyzing,
    Complete,
    Error(String),
}

fn resolve_idl_file(idl_path: PathBuf) -> Result<PathBuf> {
    if idl_path.is_dir() {
        let entries = fs::read_dir(&idl_path)
            .with_context(|| format!("Failed to read IDL directory: {:?}", idl_path))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                return Ok(path);
            }
        }
        
        anyhow::bail!("No JSON IDL file found in directory: {:?}", idl_path);
    } else {
        Ok(idl_path)
    }
}

pub async fn execute(idl_path: PathBuf, output: PathBuf, rpc_url: &str, off_chain: bool) -> Result<()> {
    info!("Starting test generation process...");

    let resolved_idl_path = resolve_idl_file(idl_path)?;
    info!("Using IDL file: {:?}", resolved_idl_path);

    let idl_data = parse_idl(&resolved_idl_path).with_context(||
        format!("Failed to parse IDL file: {:?}", resolved_idl_path)
    )?;

    let program_id = get_program_id(&resolved_idl_path)?;

    let execution_order: Vec<String> = {
        let instruction_names: Vec<String> = idl_data.instructions
            .iter()
            .map(|i| i.name.clone())
            .collect();
        select_instruction_order_interactive(&instruction_names)?
    };


    for instr_name in &execution_order {
        if !idl_data.instructions.iter().any(|i| &i.name == instr_name) {
            anyhow::bail!("Instruction '{}' not found in IDL", instr_name);
        }
    }

    let wallet_path = {
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to your wallet keypair")
            .default("~/.config/solana/id.json".to_string())
            .interact_text()?;
        PathBuf::from(shellexpand::tilde(&path).to_string())
    };

    let paraphrase = {
        let paraphrase: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter paraphrase for test metadata")
            .default("updated".to_string())
            .interact_text()?;
        paraphrase
    };

    let anchor_test_dir = detect_anchor_test_directory(&resolved_idl_path)?;

    run_interactive_test_generation(
        &idl_data,
        &execution_order,
        &program_id,
        &wallet_path,
        &output,
        &anchor_test_dir,
        rpc_url,
        &paraphrase,
        off_chain
    ).await?;

    Ok(())
}

async fn run_interactive_test_generation(
    idl_data: &solify_common::IdlData,
    execution_order: &[String],
    program: &str,
    wallet_path: &PathBuf,
    output: &PathBuf,
    anchor_test_dir: &Option<PathBuf>,
    rpc_url: &str,
    paraphrase: &str,
    off_chain: bool
) -> Result<()> {
    let mut terminal = init_terminal()?;
    let event_handler = EventHandler::new(Duration::from_millis(100));

    let mut state = AppState::Analyzing;
    let mut progress = 0.0;
    let mut test_metadata: Option<TestMetadata> = None;
    let mut error_msg: Option<String> = None;
    let mut test_files_generated = false;

    let idl_clone = idl_data.clone();
    let execution_order_clone = execution_order.to_vec();
    let program_clone = program.to_string();
    let rpc_url_clone = rpc_url.to_string();
    let wallet_clone = wallet_path.clone();
    let paraphrase_clone = paraphrase.to_string();

    let mut onchain_handle = if off_chain {
        // Use off-chain computation
        Some(tokio::spawn(async move {
            process_offchain(
                &idl_clone,
                &execution_order_clone,
                &program_clone
            )
        }))
    } else {
        // Use on-chain computation
        Some(tokio::spawn(async move {
            process_onchain(
                &idl_clone,
                &execution_order_clone,
                &program_clone,
                &rpc_url_clone,
                &wallet_clone,
                &paraphrase_clone
            ).await
        }))
    };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                    Constraint::Length(5),
                ])
                .split(f.area());

            let banner_msg = if off_chain {
                "Processing off-chain (local computation)..."
            } else {
                "Processing on-chain with Solify program..."
            };
            render_banner(
                f,
                chunks[0],
                "Generating Test Metadata",
                Some(banner_msg)
            );

            render_progress(f, chunks[1], "Generating Test Metadata", progress);

            match &state {
                AppState::Complete => {
                    if let Some(ref metadata) = test_metadata {
                        let mut info = vec![
                            "✓ Test metadata generated on-chain".to_string(),
                        ];

                        if test_files_generated {
                            let final_output = if let Some(anchor_dir) = anchor_test_dir {
                                anchor_dir.display().to_string()
                            } else {
                                output.display().to_string()
                            };
                            info.push("✓ Test files generated!".to_string());
                            info.push(format!("  Location: {}", final_output));
                        } else {
                            info.push("⏳ Generating test files...".to_string());
                        }

                        info.extend(
                            vec![
                                "".to_string(),
                                format!(
                                    "Account dependencies: {}",
                                    metadata.account_dependencies.len()
                                ),
                                format!("PDAs detected: {}", metadata.pda_init_sequence.len()),
                                format!(
                                    "Setup requirements: {}",
                                    metadata.setup_requirements.len()
                                ),
                                format!("Total Instructions: {}", metadata.instruction_order.len()),
                                format!(
                                    "Total test cases: {}",
                                    metadata.test_cases.iter().map(|tc| tc.positive_cases.len() + tc.negative_cases.len()).sum::<usize>()
                                )
                            ]
                        );
                        render_info_box(f, chunks[2], "Results", info);
                    }
                }
                AppState::Error(err) => {
                    render_info_box(f, chunks[2], "Error", vec![format!("Error: {}", err)]);
                }
                _ => {}
            }

            let status_msg = match &state {
                AppState::Analyzing => "Analyzing...",
                AppState::Complete => "Complete!",
                AppState::Error(_) => "Error occurred",
                _ => "Unknown state",
            };
            render_status(f, chunks[3], status_msg, matches!(state, AppState::Error(_)));
            render_key_hints(f, chunks[4], vec![("q", "Quit"), ("Enter", "Continue")]);
        })?;

        if matches!(state, AppState::Analyzing) {
            progress = (progress + 0.01).min(0.99);
        }

        if matches!(state, AppState::Analyzing) {
            if let Some(handle) = &onchain_handle {
                if handle.is_finished() {
                    if let Some(handle) = onchain_handle.take() {
                        match handle.await {
                            Ok(Ok(metadata)) => {
                                progress = 1.0;
                                test_metadata = Some(metadata.clone());
                                state = AppState::Complete;
                                if !test_files_generated {
                                    test_files_generated = true;
                                    let final_output = if let Some(anchor_dir) = anchor_test_dir {
                                        anchor_dir.clone()
                                    } else {
                                        output.clone()
                                    };
                                    if let Err(e) = fs::create_dir_all(&final_output) {
                                        error_msg = Some(
                                            format!(
                                                "Failed to create output directory: {:?}: {}",
                                                final_output,
                                                e
                                            )
                                        );
                                        state = AppState::Error(error_msg.as_ref().unwrap().clone());
                                    } else {
                                        match generate_with_tera(&metadata, &idl_data, &final_output) {
                                            Ok(_) => {
                                                info!("Test files generated successfully!");
                                            }
                                            Err(e) => {
                                                error_msg = Some(
                                                    format!("Failed to generate test files: {}", e)
                                                );
                                                state = AppState::Error(
                                                    error_msg.as_ref().unwrap().clone()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                if is_program_too_large_error(&e) {
                                    error_msg = Some(
                                        "Your Anchor program is too large for on-chain processing.\n\
                                        The IDL data or test metadata generation exceeds the available compute units or memory limits.\n\
                                        Please wait for the next updates to generate tests for your program.".to_string()
                                    );
                                } else {
                                    error_msg = Some(format!("On-chain processing failed: {}", e));
                                }
                                state = AppState::Error(error_msg.as_ref().unwrap().clone());
                                progress = 0.0;
                            }
                            Err(e) => {
                                error_msg = Some(format!("Task execution failed: {}", e));
                                state = AppState::Error(error_msg.as_ref().unwrap().clone());
                                progress = 0.0;
                            }
                        }
                    }
                }
            }
        }

        match event_handler.next()? {
            AppEvent::Quit => {
                restore_terminal(terminal)?;
                if let Some(err) = error_msg {
                    anyhow::bail!("Analysis failed: {}", err);
                }
                return Ok(());
            }
            AppEvent::Enter => {
                if matches!(state, AppState::Complete | AppState::Error(_)) {
                    break;
                }
            }
            _ => {}
        }
    }

    restore_terminal(terminal)?;

    if let Some(metadata) = test_metadata {
        println!("\n✅ On-chain processing complete!");
        if test_files_generated {
            let final_output = if let Some(anchor_dir) = anchor_test_dir {
                anchor_dir
            } else {
                output
            };

            let idl_name = sanitize_idl_name(&idl_data.name);
            let test_file = final_output.join(format!("{}.test.ts", idl_name));
            if test_file.exists() {
                println!("Test file: {}", test_file.display());
            }
            println!("\n   Run `anchor test` to execute the tests");
        } else {
            let final_output = if let Some(anchor_dir) = anchor_test_dir {
                println!("\n📁 Detected Anchor project structure");
                println!("   Saving tests to: {}", anchor_dir.display());
                anchor_dir.clone()
            } else {
                output.clone()
            };
            fs
                ::create_dir_all(&final_output)
                .with_context(|| format!("Failed to create output directory: {:?}", final_output))?;

            println!("\n📝 Generating TypeScript test files...");
            println!("   Output directory: {}", final_output.display());
            println!("   IDL name: {}", idl_data.name);

            generate_with_tera(&metadata, &idl_data, &final_output).with_context(||
                format!("Failed to generate test files in: {:?}", final_output)
            )?;
        }
    }

    if let Some(err) = error_msg {
        // Check if it's already a user-friendly message or if we need to check the error
        if err.contains("too large for on-chain processing") {
            anyhow::bail!("{}", err);
        } else {
            anyhow::bail!("On-chain processing failed: {}", err);
        }
    }

    Ok(())
}

fn process_offchain(
    idl_data: &solify_common::IdlData,
    execution_order: &Vec<String>,
    program: &str,
) -> Result<TestMetadata> {
    let analyzer = DependencyAnalyzer::new();
    analyzer.analyze_dependencies(idl_data, execution_order, program.to_string())
        .map_err(|e| anyhow::anyhow!("Off-chain analysis failed: {}", e))
}

fn is_program_too_large_error(err: &anyhow::Error) -> bool {
    // Check the full error chain (including context messages)
    let err_str = err.to_string().to_lowercase();
    
    // Also check the root cause by iterating through the error chain
    let mut current_err: Option<&dyn std::error::Error> = Some(err.as_ref());
    let mut all_error_messages = vec![err_str];
    
    while let Some(e) = current_err {
        all_error_messages.push(e.to_string().to_lowercase());
        current_err = e.source();
    }
    
    // Check all error messages in the chain
    for msg in &all_error_messages {
        if msg.contains("out of memory") ||
           msg.contains("memory allocation failed") ||
           (msg.contains("sbf program panicked") && 
            (msg.contains("memory") || msg.contains("allocation") || 
             msg.contains("out of"))) ||
           (msg.contains("program failed to complete") && 
            (msg.contains("memory") || msg.contains("allocation") || 
             msg.contains("out of"))) ||
           msg.contains("compute budget exceeded") ||
           msg.contains("insufficient compute units") ||
           // Check transaction simulation failures with memory-related issues
           (msg.contains("transaction simulation failed") &&
            (msg.contains("memory") || msg.contains("allocation") || 
             msg.contains("out of") || 
             (msg.contains("panicked") && (msg.contains("memory") || msg.contains("allocation"))))) ||
           // Also check for generic transaction failures that might indicate size issues
           (msg.contains("failed to send solify transaction") &&
            (msg.contains("memory") || msg.contains("allocation") || 
             msg.contains("panicked") || msg.contains("out of")))
        {
            return true;
        }
    }
    
    false
}

async fn process_onchain(
    idl_data: &solify_common::IdlData,
    execution_order: &Vec<String>,
    program: &str,
    rpc_url: &str,
    wallet_path: &PathBuf,
    paraphrase: &str
) -> Result<TestMetadata> {
    let wallet_data = fs::read_to_string(&wallet_path)
        .with_context(|| format!("Failed to read wallet file: {:?}", wallet_path))?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)
        .with_context(|| format!("Failed to parse wallet JSON: {:?}", wallet_path))?;
    
    if wallet_bytes.len() < 32 {
        return Err(anyhow::anyhow!(
            "Invalid wallet keypair: expected at least 32 bytes, got {}",
            wallet_bytes.len()
        ));
    }
    
    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&wallet_bytes[..32]);
    let wallet_keypair = Keypair::new_from_array(secret_key);

    let user_pubkey = wallet_keypair.pubkey();

    let program_id = Pubkey::from_str(&program)
        .with_context(|| format!("Invalid program ID: {}", program))?;
    let client = SolifyClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
        .with_context(|| format!("Failed to create Solify client for RPC: {}", rpc_url))?;

    let idl_storage = client.fetch_idl_storage(user_pubkey, program_id)
        .with_context(|| "Failed to fetch IDL storage account")?;
    if idl_storage.is_some() {
        let update_result = client.update_idl_data(&wallet_keypair, program_id, &idl_data)
            .with_context(|| "Failed to update IDL data on-chain");
        
        if let Err(ref e) = update_result {
            if is_program_too_large_error(e) {
                return Err(anyhow::anyhow!(
                    "Your Anchor program is too large for on-chain processing.\n\
                    The IDL data exceeds the available compute units or memory limits.\n\
                    Please wait for the next updates to generate tests for your program.\n\
                    \n\
                    Error details: {}",
                    e
                ));
            }
        }
        
        let _update_idl_sig = update_result?;

        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let idl_storage = client.fetch_idl_storage(user_pubkey, program_id)
            .with_context(|| "Failed to verify IDL storage after update")?;
        if idl_storage.is_none() {
            return Err(anyhow::anyhow!(
                "IDL storage account not found after update. The update transaction may have failed. \
                Please check the transaction signature and verify the program is deployed correctly."
            ));
        }
        
        let existing_metadata = client.fetch_test_metadata(user_pubkey, program_id, paraphrase)
            .with_context(|| "Failed to check for existing test metadata")?;
        if existing_metadata.is_none() {
            let generate_result = client.generate_metadata(
                &wallet_keypair,
                program_id,
                execution_order.clone(),
                paraphrase,
                program.to_string()
            ).with_context(|| "Failed to generate test metadata on-chain");
            
            if let Err(ref e) = generate_result {
                if is_program_too_large_error(e) {
                    return Err(anyhow::anyhow!(
                        "Your Anchor program is too large for on-chain processing.\n\
                        The test metadata generation exceeds the available compute units or memory limits.\n\
                        Please wait for the next updates to generate tests for your program.\n\
                        \n\
                        Error details: {}",
                        e
                    ));
                }
            }
            
            let _test_metadata_sig = generate_result?;
        } else {
            let update_result = client.generate_metadata(
                &wallet_keypair,
                program_id,
                execution_order.clone(),
                paraphrase,
                program.to_string()
            ).with_context(|| "Failed to update test metadata on-chain");
            
            if let Err(ref e) = update_result {
                if is_program_too_large_error(e) {
                    return Err(anyhow::anyhow!(
                        "Your Anchor program is too large for on-chain processing.\n\
                        The test metadata generation exceeds the available compute units or memory limits.\n\
                        Please wait for the next updates to generate tests for your program.\n\
                        \n\
                        Error details: {}",
                        e
                    ));
                }
            }
            
            let _update_test_metadata_sig = update_result?;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let test_metadata_account = client.fetch_test_metadata(
            user_pubkey,
            program_id,
            paraphrase
        ).with_context(|| "Failed to fetch test metadata from on-chain account")?;
        
        match test_metadata_account {
            Some(account) => Ok(account.test_metadata),
            None => {
                Err(anyhow::anyhow!(
                    "Test metadata account not found. The transaction may have failed. \
                    Please check the transaction signature and verify the program is deployed correctly."
                ))
            }
        }
    } else {
        let store_result = client.store_idl_data(&wallet_keypair, program_id, &idl_data)
            .with_context(|| "Failed to store IDL data on-chain");
        
        if let Err(ref e) = store_result {
            if is_program_too_large_error(e) {
                return Err(anyhow::anyhow!(
                    "Your Anchor program is too large for on-chain processing.\n\
                    The IDL data exceeds the available compute units or memory limits.\n\
                    Please wait for the next updates to generate tests for your program.\n\
                    \n\
                    Error details: {}",
                    e
                ));
            }
        }
        
        let _store_idl_sig = store_result?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let generate_result = client.generate_metadata(
            &wallet_keypair,
            program_id,
            execution_order.clone(),
            paraphrase,
            program.to_string()
        ).with_context(|| "Failed to generate test metadata on-chain");
        
        if let Err(ref e) = generate_result {
            if is_program_too_large_error(e) {
                return Err(anyhow::anyhow!(
                    "Your Anchor program is too large for on-chain processing.\n\
                    The test metadata generation exceeds the available compute units or memory limits.\n\
                    Please wait for the next updates to generate tests for your program.\n\
                    \n\
                    Error details: {}",
                    e
                ));
            }
        }
        
        let _test_metadata_sig = generate_result?;
    
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let test_metadata_account = client.fetch_test_metadata(
            user_pubkey,
            program_id,
            paraphrase
        ).with_context(|| "Failed to fetch test metadata from on-chain account")?;
        
        match test_metadata_account {
            Some(account) => Ok(account.test_metadata),
            None => {
                Err(anyhow::anyhow!(
                    "Test metadata account not found. The transaction may have failed. \
                    Please check the transaction signature and verify the program is deployed correctly."
                ))
            }
        }
    }
}

fn detect_anchor_test_directory(idl_path: &PathBuf) -> Result<Option<PathBuf>> {
    let idl_parent = idl_path.parent();
    if let Some(parent) = idl_parent {
        let parent_str = parent.to_string_lossy();

        if parent_str.contains("target") && parent_str.contains("idl") {
            if let Some(grandparent) = parent.parent() {
                if let Some(project_root) = grandparent.parent() {
                    let test_dir = project_root.join("tests");
                    if !test_dir.exists() {
                        fs
                            ::create_dir_all(&test_dir)
                            .with_context(||
                                format!("Failed to create tests directory: {:?}", test_dir)
                            )?;
                    }
                    return Ok(Some(test_dir));
                }
            }
        }
    }

    Ok(None)
}

fn sanitize_idl_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
                _ => '_',
            }
        })
        .collect()
}

fn select_instruction_order_interactive(instructions: &[String]) -> Result<Vec<String>> {
    let mut terminal = init_terminal()?;
    let event_handler = EventHandler::new(Duration::from_millis(100));

    let mut selected_instructions: Vec<String> = Vec::new();
    let mut available_instructions = instructions.to_vec();
    let mut cursor = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                    Constraint::Length(5),
                ])
                .split(f.area());

            render_banner(
                f,
                chunks[0],
                "Select Instruction Execution Order",
                Some("Use ↑/↓ to navigate, Enter to select, 'd' to finish")
            );
            let middle_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            render_list(
                f,
                middle_chunks[0],
                "Available Instructions",
                available_instructions.clone(),
                Some(cursor)
            );
            render_list(f, middle_chunks[1], "Selected Order", selected_instructions.clone(), None);

            let status_msg = format!(
                "Selected {}/{} instructions",
                selected_instructions.len(),
                instructions.len()
            );
            render_status(f, chunks[2], &status_msg, false);
            render_key_hints(
                f,
                chunks[3],
                vec![("↑/↓", "Navigate"), ("Enter", "Select"), ("d", "Done"), ("q", "Quit")]
            );
        })?;

        match event_handler.next()? {
            AppEvent::Quit => {
                restore_terminal(terminal)?;
                anyhow::bail!("User cancelled");
            }
            AppEvent::Up => {
                if cursor > 0 {
                    cursor -= 1;
                }
            }
            AppEvent::Down => {
                if cursor < available_instructions.len().saturating_sub(1) {
                    cursor += 1;
                }
            }
            AppEvent::Enter => {
                if !available_instructions.is_empty() && cursor < available_instructions.len() {
                    let selected = available_instructions.remove(cursor);
                    selected_instructions.push(selected);
                    if cursor >= available_instructions.len() && cursor > 0 {
                        cursor -= 1;
                    }
                }
            }
            AppEvent::Char('d') | AppEvent::Char('D') => {
                if !selected_instructions.is_empty() {
                    break;
                }
            }
            _ => {}
        }
    }

    restore_terminal(terminal)?;
    Ok(selected_instructions)
}
