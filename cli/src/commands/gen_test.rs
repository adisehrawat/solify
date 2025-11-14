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

pub async fn execute(idl_path: PathBuf, output: PathBuf, rpc_url: &str) -> Result<()> {
    info!("Starting test generation process...");

    let idl_data = parse_idl(&idl_path).with_context(||
        format!("Failed to parse IDL file: {:?}", idl_path)
    )?;
    info!("Successfully parsed IDL: {}", idl_data.name);
    info!("Found {} instructions", idl_data.instructions.len());

    let program_id = get_program_id(&idl_path)?;
    info!("Program ID: {}", program_id);

    let execution_order: Vec<String> = {
        let instruction_names: Vec<String> = idl_data.instructions
            .iter()
            .map(|i| i.name.clone())
            .collect();
        select_instruction_order_interactive(&instruction_names)?
    };

    info!("Execution order: {:?}", execution_order);

    for instr_name in &execution_order {
        if !idl_data.instructions.iter().any(|i| &i.name == instr_name) {
            anyhow::bail!("Instruction '{}' not found in IDL", instr_name);
        }
    }

    // get wallet path
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

    let anchor_test_dir = detect_anchor_test_directory(&idl_path)?;

    run_interactive_test_generation(
        &idl_data,
        &execution_order,
        &program_id,
        &wallet_path,
        &output,
        &anchor_test_dir,
        rpc_url,
        &paraphrase
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
    paraphrase: &str
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

    let mut onchain_handle = Some(
        tokio::spawn(async move {
            process_onchain(
                &idl_clone,
                &execution_order_clone,
                &program_clone,
                &rpc_url_clone,
                &wallet_clone,
                &paraphrase_clone
            ).await
        })
    );

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

            render_banner(
                f,
                chunks[0],
                "Generating Test Metadata",
                Some("Waiting for on-chain operations to complete...")
            );

            render_progress(f, chunks[1], "Generating Test Metadata", progress);

            match &state {
                AppState::Analyzing => {
                    render_info_box(
                        f,
                        chunks[2],
                        "Status",
                        vec![
                            "Processing on-chain with Solify program...".to_string(),
                            "Step 1: Initializing user account".to_string(),
                            "Step 2: Storing IDL data".to_string(),
                            "Step 3: Generating test metadata".to_string(),
                            "Step 4: Fetching results from PDA".to_string(),
                            "".to_string(),
                            format!("Instructions: {}", execution_order.len()),
                            format!("Program: {}", program)
                        ]
                    );
                }
                AppState::Complete => {
                    if let Some(ref metadata) = test_metadata {
                        let mut info = vec![
                            "✓ On-chain processing complete!".to_string(),
                            "✓ Test metadata fetched from PDA".to_string()
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
                                    "Positive cases: {}",
                                    metadata.test_cases
                                        .iter()
                                        .map(|tc| tc.positive_cases.len())
                                        .sum::<usize>()
                                ),
                                format!(
                                    "Negative cases: {}",
                                    metadata.test_cases
                                        .iter()
                                        .map(|tc| tc.negative_cases.len())
                                        .sum::<usize>()
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

            // Key hints
            render_key_hints(f, chunks[4], vec![("q", "Quit"), ("Enter", "Continue")]);
        })?;

        if matches!(state, AppState::Analyzing) {
            progress = (progress + 0.01).min(0.99);
        }

        if matches!(state, AppState::Analyzing) {
            if let Some(handle) = &onchain_handle {
                if handle.is_finished() {
                    // Take ownership and await
                    if let Some(handle) = onchain_handle.take() {
                        if let Ok(Ok(metadata)) = handle.await {
                            progress = 1.0;
                            test_metadata = Some(metadata.clone());
                            state = AppState::Complete;

                            // Generate test files automatically when on-chain processing completes
                            if !test_files_generated {
                                test_files_generated = true;

                                // Determine output directory
                                let final_output = if let Some(anchor_dir) = anchor_test_dir {
                                    anchor_dir.clone()
                                } else {
                                    output.clone()
                                };

                                // Ensure output directory exists
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
                                    // Generate test files
                                    match generate_with_tera(&metadata, &idl_data, &final_output) {
                                        Ok(_) => {
                                            info!("Test files generated successfully!");
                                            info!("Output directory: {}", final_output.display());
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

        // If test files were already generated in the loop, just show summary
        if test_files_generated {
            let final_output = if let Some(anchor_dir) = anchor_test_dir {
                println!("\n📁 Detected Anchor project structure");
                println!("   Tests saved to: {}", anchor_dir.display());
                anchor_dir
            } else {
                println!("\n📁 Tests saved to: {}", output.display());
                output
            };

            // Verify files were created
            let idl_name = sanitize_idl_name(&idl_data.name);
            let test_file = final_output.join(format!("{}.test.ts", idl_name));
            if test_file.exists() {
                println!("  ✅ Test file: {}", test_file.display());
            }
            println!("\n   Run `anchor test` to execute the tests");
        } else {
            // Fallback: Generate test files here if they weren't generated in the loop
            let final_output = if let Some(anchor_dir) = anchor_test_dir {
                println!("\n📁 Detected Anchor project structure");
                println!("   Saving tests to: {}", anchor_dir.display());
                anchor_dir.clone()
            } else {
                output.clone()
            };

            // Ensure output directory exists
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
        anyhow::bail!("On-chain processing failed: {}", err);
    }

    Ok(())
}

async fn process_onchain(
    idl_data: &solify_common::IdlData,
    execution_order: &Vec<String>,
    program: &str,
    rpc_url: &str,
    wallet_path: &PathBuf,
    paraphrase: &str
) -> Result<TestMetadata> {
    let wallet_data = fs::read_to_string(&wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let mut secret_key = [0u8; 32];
    secret_key.copy_from_slice(&wallet_bytes[..32]);
    let wallet_keypair = Keypair::new_from_array(secret_key);

    let user_pubkey = wallet_keypair.pubkey();

    let program_id = Pubkey::from_str(&program)?;
    let client = SolifyClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())?;

    println!("  ⏳ Step 1/4: checking if IDL data is already stored on-chain...");
    let idl_storage = client.fetch_idl_storage(user_pubkey, program_id)?;
    if idl_storage.is_some() {
        println!("  ✅ IDL data is already stored on-chain");
        println!(" Step 3/4: Updating IDL data on-chain...");
        let update_idl_sig = client.update_idl_data(&wallet_keypair, program_id, &idl_data)?;
        println!("  ✅ IDL data updated on-chain");
        println!(" Signature for updating IDL data: {}", update_idl_sig);
        let idl_storage = client.fetch_idl_storage(user_pubkey, program_id)?;
        if idl_storage.is_some() {
            println!("  ✅ IDL data updated on-chain");
        } else {
            return Err(anyhow::anyhow!("Failed to fetch IDL data after operation"));
        }
        println!(" Step 4/4: checking if test metadata exists...");
        let existing_metadata = client.fetch_test_metadata(user_pubkey, program_id, paraphrase)?;

        if existing_metadata.is_some() {
            println!("  ✅ Test metadata exists on-chain");
            println!(" Step 4/4: Generating new test metadata on-chain...");
            let update_test_metadata_sig = client.generate_metadata(
                &wallet_keypair,
                program_id,
                execution_order.clone(),
                paraphrase,
                program.to_string()
            )?;
            println!("  ✅ Test metadata updated on-chain");
            println!(" Signature for updating test metadata: {}", update_test_metadata_sig);
        } else {
            println!("  ⏳ Step 4/4: generating test metadata on-chain...");
            let test_metadata_sig = client.generate_metadata(
                &wallet_keypair,
                program_id,
                execution_order.clone(),
                paraphrase,
                program.to_string()
            )?;
            println!("  ✅ Test metadata generated on-chain");
            println!(" Signature for generating test metadata: {}", test_metadata_sig);
        }
        let test_metadata_account = client.fetch_test_metadata(
            user_pubkey,
            program_id,
            paraphrase
        )?;
        if let Some(test_metadata_account) = test_metadata_account {
            return Ok(test_metadata_account.test_metadata);
        } else {
            return Err(anyhow::anyhow!("Failed to fetch test metadata account after operation"));
        }
    } else {
        println!("  ⏳ Step 3/4: storing IDL data on-chain...");
        let store_idl_sig = client.store_idl_data(&wallet_keypair, program_id, idl_data)?;
        println!("  ✅ IDL data stored on-chain");
        println!(" Signature for storing IDL data: {}", store_idl_sig);
        println!("  ⏳ Step 4/4: generating test metadata on-chain...");
        let test_metadata_sig = client.generate_metadata(
            &wallet_keypair,
            program_id,
            execution_order.clone(),
            paraphrase,
            program.to_string()
        )?;
        println!("  ✅ Test metadata generated on-chain");
        println!(" Signature for generating test metadata: {}", test_metadata_sig);
        let test_metadata_account = client.fetch_test_metadata(
            user_pubkey,
            program_id,
            paraphrase
        )?;
        if let Some(test_metadata_account) = test_metadata_account {
            return Ok(test_metadata_account.test_metadata);
        } else {
            return Err(anyhow::anyhow!("Failed to fetch test metadata account after operation"));
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
                    // Create the tests directory if it doesn't exist
                    if !test_dir.exists() {
                        fs
                            ::create_dir_all(&test_dir)
                            .with_context(||
                                format!("Failed to create tests directory: {:?}", test_dir)
                            )?;
                        info!("Created Anchor tests directory: {:?}", test_dir);
                    } else {
                        info!("Detected Anchor tests directory: {:?}", test_dir);
                    }
                    return Ok(Some(test_dir));
                }
            }
        }
    }

    Ok(None)
}

/// Sanitize IDL name for use in filenames
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

            // Banner
            render_banner(
                f,
                chunks[0],
                "Select Instruction Execution Order",
                Some("Use ↑/↓ to navigate, Enter to select, 'd' to finish")
            );

            // Split middle section for available and selected
            let middle_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Available instructions
            render_list(
                f,
                middle_chunks[0],
                "Available Instructions",
                available_instructions.clone(),
                Some(cursor)
            );

            // Selected instructions
            render_list(f, middle_chunks[1], "Selected Order", selected_instructions.clone(), None);

            // Status
            let status_msg = format!(
                "Selected {}/{} instructions",
                selected_instructions.len(),
                instructions.len()
            );
            render_status(f, chunks[2], &status_msg, false);

            // Key hints
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
