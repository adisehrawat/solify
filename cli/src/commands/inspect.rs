use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedTransaction,
    UiInstruction,
    UiMessage,
    UiParsedInstruction,
    UiTransactionEncoding,
    UiTransactionTokenBalance,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use std::collections::HashMap;
use std::str::FromStr;
use serde_json::Value;

use crate::tui::{init_terminal, restore_terminal, EventHandler, AppEvent};
use crate::tui::widgets::{
    render_banner, render_info_box, render_scrollable_info_box, render_status,
};
use log::info;

pub async fn execute(signature: String, rpc_url: &str) -> Result<()> {
    info!("Inspecting transaction: {}", signature);
    
    match inspect_transaction_interactive(&signature, rpc_url).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("Device not configured") || 
                  e.to_string().contains("not a terminal") => {
            info!("Terminal not available, using simple output mode");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

struct TransactionDetails {
    signature: String,
    slot: u64,
    block_time: String,
    status: String,
    fee: u64,
    instructions: Vec<InstructionInfo>,
    accounts: Vec<AccountInfo>,
    logs: Vec<String>,
    compute_units: Option<u64>,
    return_data: Option<ReturnDataInfo>,
}

struct InstructionInfo {
    program_title: String,
    instruction_summary: Vec<String>,
}

struct AccountInfo {
    pubkey: String,
    pre_balance: u64,
    post_balance: u64,
    is_signer: bool,
    is_writable: bool,
    source: Option<String>,
    token_balances: Vec<String>,
}

struct ReturnDataInfo {
    program_id: String,
    data_base64: String,
}

fn option_serializer_to_option<T: Clone>(value: &OptionSerializer<T>) -> Option<T> {
    match value {
        OptionSerializer::Some(data) => Some(data.clone()),
        _ => None,
    }
}

fn option_serializer_to_vec<T: Clone>(value: &OptionSerializer<Vec<T>>) -> Vec<T> {
    match value {
        OptionSerializer::Some(items) => items.clone(),
        _ => Vec::new(),
    }
}

fn format_simple_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn format_json_value(value: &Value, indent: usize, max_lines: usize) -> Vec<String> {
    fn helper(value: &Value, indent: usize, max_lines: usize, lines: &mut Vec<String>) {
        if lines.len() >= max_lines {
            return;
        }
        let pad = " ".repeat(indent);
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter() {
                    if lines.len() >= max_lines {
                        break;
                    }
                    match val {
                        Value::Object(_) | Value::Array(_) => {
                            lines.push(format!("{}{}:", pad, key));
                            helper(val, indent + 2, max_lines, lines);
                        }
                        _ => {
                            lines.push(format!(
                                "{}{}: {}",
                                pad,
                                key,
                                format_simple_json_value(val)
                            ));
                        }
                    }
                }
            }
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    if lines.len() >= max_lines {
                        break;
                    }
                    match item {
                        Value::Object(_) | Value::Array(_) => {
                            lines.push(format!("{}[{}]:", pad, index));
                            helper(item, indent + 2, max_lines, lines);
                        }
                        _ => {
                            lines.push(format!(
                                "{}- {}",
                                pad,
                                format_simple_json_value(item)
                            ));
                        }
                    }
                }
            }
            _ => lines.push(format!("{}{}", pad, format_simple_json_value(value))),
        }
    }

    let mut lines = Vec::new();
    helper(value, indent, max_lines, &mut lines);
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        lines.push(format!("{}...", " ".repeat(indent)));
    }
    lines
}

fn build_token_balance_map(
    pre: &[UiTransactionTokenBalance],
    post: &[UiTransactionTokenBalance],
) -> HashMap<u8, Vec<String>> {
    let mut result: HashMap<u8, Vec<String>> = HashMap::new();
    let mut pre_map: HashMap<(u8, String), UiTransactionTokenBalance> = HashMap::new();

    for balance in pre {
        pre_map.insert(
            (balance.account_index, balance.mint.clone()),
            balance.clone(),
        );
    }

    for balance in post {
        let key = (balance.account_index, balance.mint.clone());
        let previous = pre_map.remove(&key);

        let pre_amount = previous
            .as_ref()
            .map(|b| b.ui_token_amount.real_number_string_trimmed())
            .unwrap_or_else(|| "0".to_string());
        let pre_value = previous
            .as_ref()
            .and_then(|b| b.ui_token_amount.ui_amount)
            .unwrap_or(0.0);

        let post_amount = balance.ui_token_amount.real_number_string_trimmed();
        let post_value = balance.ui_token_amount.ui_amount.unwrap_or(0.0);
        let delta = post_value - pre_value;
        let delta_str = if delta >= 0.0 {
            format!("+{:.6}", delta)
        } else {
            format!("{:.6}", delta)
        };

        result
            .entry(balance.account_index)
            .or_default()
            .push(format!(
                "Token {}: {} → {} (Δ {})",
                balance.mint,
                pre_amount,
                post_amount,
                delta_str.trim_end_matches('0').trim_end_matches('.')
            ));
    }

    for ((account_index, mint), previous) in pre_map {
        let pre_amount = previous.ui_token_amount.real_number_string_trimmed();
        result
            .entry(account_index)
            .or_default()
            .push(format!("Token {}: {} → 0", mint, pre_amount));
    }

    result
}

fn format_instruction_lines(
    instruction: &UiInstruction,
    accounts: &[AccountInfo],
    indent: usize,
) -> Vec<String> {
    let pad = " ".repeat(indent);
    match instruction {
        UiInstruction::Parsed(parsed) => match parsed {
            UiParsedInstruction::Parsed(parsed_instr) => {
                let mut lines = vec![format!(
                    "{}Program: {} ({})",
                    pad, parsed_instr.program, parsed_instr.program_id
                )];

                match &parsed_instr.parsed {
                    Value::Object(obj) => {
                        if let Some(Value::String(kind)) = obj.get("type") {
                            lines.push(format!("{}  Type: {}", pad, kind));
                        }
                        if let Some(info) = obj.get("info") {
                            lines.push(format!("{}  Info:", pad));
                            lines.extend(format_json_value(info, indent + 4, 16));
                        } else {
                            lines.extend(format_json_value(&parsed_instr.parsed, indent + 2, 16));
                        }
                    }
                    value => {
                        lines.push(format!(
                            "{}  {}",
                            pad,
                            format_simple_json_value(value)
                        ));
                    }
                }

                lines
            }
            UiParsedInstruction::PartiallyDecoded(decoded) => {
                let mut lines =
                    vec![format!("{}Program ID: {}", pad, decoded.program_id)];
                if !decoded.accounts.is_empty() {
                    lines.push(format!("{}  Accounts:", pad));
                    for account in &decoded.accounts {
                        lines.push(format!("{}    {}", pad, account));
                    }
                }
                if !decoded.data.is_empty() {
                    lines.push(format!("{}  Data (base58): {}", pad, decoded.data));
                }
                lines
            }
        },
        UiInstruction::Compiled(compiled) => {
            let mut lines = Vec::new();
            let program_name = accounts
                .get(compiled.program_id_index as usize)
                .map(|account| account.pubkey.clone())
                .unwrap_or_else(|| format!("Program index {}", compiled.program_id_index));
            lines.push(format!("{}Program: {}", pad, program_name));
            lines.push(format!(
                "{}  Program index: {}",
                pad, compiled.program_id_index
            ));
            if !compiled.accounts.is_empty() {
                lines.push(format!("{}  Accounts:", pad));
                for account_index in &compiled.accounts {
                    if let Some(account) = accounts.get(*account_index as usize) {
                        lines.push(format!(
                            "{}    [{}] {}",
                            pad, account_index, account.pubkey
                        ));
                    } else {
                        lines.push(format!("{}    [{}] <unknown>", pad, account_index));
                    }
                }
            }
            if !compiled.data.is_empty() {
                lines.push(format!("{}  Data (base58): {}", pad, compiled.data));
            }
            lines
        }
    }
}

async fn inspect_transaction_interactive(
    signature_str: &str,
    rpc_url: &str,
) -> Result<()> {
    let mut terminal = init_terminal()?;
    let event_handler = EventHandler::new(Duration::from_millis(100));

    let client = RpcClient::new(rpc_url.to_string());
    let signature = Signature::from_str(signature_str)?;

    info!("Fetching transaction from RPC...");


    let tx_result = client.get_transaction_with_config(
        &signature,
        solana_client::rpc_config::RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: Some(0),
        }
    );

    let (tx_details, error_msg) = match tx_result {
        Ok(tx) => {
            let slot = tx.slot;
            let block_time = tx.block_time.map(|t| {
                chrono::DateTime::from_timestamp(t, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            }).unwrap_or_else(|| "Unknown".to_string());
            
            let meta = match tx.transaction.meta {
                Some(ref meta) => meta,
                _ => {
                    return Err(anyhow::anyhow!("Transaction metadata unavailable"));
                }
            };

            let fee = meta.fee;
            let status = if meta.status.is_ok() {
                "✓ Success".to_string()
            } else {
                "✗ Failed".to_string()
            };

            let mut instructions = Vec::new();
            let mut accounts = Vec::new();

            let inner_instruction_map: HashMap<usize, Vec<UiInstruction>> =
                option_serializer_to_vec(&meta.inner_instructions)
                    .into_iter()
                    .map(|inner| (inner.index as usize, inner.instructions))
                    .collect();

            let pre_token_balances = option_serializer_to_vec(&meta.pre_token_balances);
            let post_token_balances = option_serializer_to_vec(&meta.post_token_balances);
            let mut token_balance_map =
                build_token_balance_map(&pre_token_balances, &post_token_balances);

            match &tx.transaction.transaction {
                EncodedTransaction::Json(json_tx) => {
                    match &json_tx.message {
                        UiMessage::Parsed(parsed_msg) => {
                            let account_keys = &parsed_msg.account_keys;

                            for (idx, account) in account_keys.iter().enumerate() {
                                let pre_balance =
                                    meta.pre_balances.get(idx).copied().unwrap_or(0);
                                let post_balance =
                                    meta.post_balances.get(idx).copied().unwrap_or(0);

                                accounts.push(AccountInfo {
                                    pubkey: account.pubkey.clone(),
                                    pre_balance,
                                    post_balance,
                                    is_signer: account.signer,
                                    is_writable: account.writable,
                                    source: account
                                        .source
                                        .as_ref()
                                        .map(|s| format!("{:?}", s)),
                                    token_balances: token_balance_map
                                        .remove(&(idx as u8))
                                        .unwrap_or_default(),
                                });
                            }

                            for (idx, instruction) in parsed_msg.instructions.iter().enumerate() {
                                let mut lines =
                                    format_instruction_lines(instruction, &accounts, 0);
                                if lines.is_empty() {
                                    lines.push("Program: <unknown>".to_string());
                                }
                                let header = lines.remove(0);
                                let mut summary = lines;

                                if let Some(inner_list) = inner_instruction_map.get(&idx) {
                                    summary.push("  Inner Instructions:".to_string());
                                    for (inner_idx, inner_ix) in inner_list.iter().enumerate() {
                                        let mut inner_lines =
                                            format_instruction_lines(inner_ix, &accounts, 4);
                                        if let Some(first) = inner_lines.first_mut() {
                                            *first = format!(
                                                "    {}. {}",
                                                inner_idx + 1,
                                                first.trim()
                                            );
                                        }
                                        summary.extend(inner_lines);
                                    }
                                }

                                instructions.push(InstructionInfo {
                                    program_title: format!("▶ {}. {}", idx + 1, header.trim()),
                                    instruction_summary: summary,
                                });
                            }
                        }
                        UiMessage::Raw(raw_msg) => {
                            // Fallback for raw messages
                            let num_signers = raw_msg.header.num_required_signatures as usize;
                            let num_readonly_signed =
                                raw_msg.header.num_readonly_signed_accounts as usize;
                            let num_readonly_unsigned =
                                raw_msg.header.num_readonly_unsigned_accounts as usize;
                            let total_accounts = raw_msg.account_keys.len();

                            let writable_signed_threshold =
                                num_signers.saturating_sub(num_readonly_signed);
                            let writable_unsigned_threshold = (total_accounts
                                - num_signers)
                                .saturating_sub(num_readonly_unsigned);

                            for (idx, pubkey) in raw_msg.account_keys.iter().enumerate() {
                                let is_signer = idx < num_signers;
                                let is_writable = if is_signer {
                                    idx < writable_signed_threshold
                                } else {
                                    let unsigned_index = idx - num_signers;
                                    unsigned_index < writable_unsigned_threshold
                                };

                                let pre_balance =
                                    meta.pre_balances.get(idx).copied().unwrap_or(0);
                                let post_balance =
                                    meta.post_balances.get(idx).copied().unwrap_or(0);

                                accounts.push(AccountInfo {
                                    pubkey: pubkey.clone(),
                                    pre_balance,
                                    post_balance,
                                    is_signer,
                                    is_writable,
                                    source: None,
                                    token_balances: token_balance_map
                                        .remove(&(idx as u8))
                                        .unwrap_or_default(),
                                });
                            }

                            for (idx, compiled) in raw_msg.instructions.iter().enumerate() {
                                let compiled_instruction =
                                    UiInstruction::Compiled(compiled.clone());
                                let mut lines = format_instruction_lines(
                                    &compiled_instruction,
                                    &accounts,
                                    0,
                                );
                                if lines.is_empty() {
                                    lines.push("Program: <compiled>".to_string());
                                }
                                let header = lines.remove(0);
                                let mut summary = lines;
                                if let Some(inner_list) = inner_instruction_map.get(&idx) {
                                    summary.push("  Inner Instructions:".to_string());
                                    for (inner_idx, inner_ix) in inner_list.iter().enumerate() {
                                        let mut inner_lines =
                                            format_instruction_lines(inner_ix, &accounts, 4);
                                        if let Some(first) = inner_lines.first_mut() {
                                            *first = format!(
                                                "    {}. {}",
                                                inner_idx + 1,
                                                first.trim()
                                            );
                                        }
                                        summary.extend(inner_lines);
                                    }
                                }

                                instructions.push(InstructionInfo {
                                    program_title: format!("▶ {}. {}", idx + 1, header.trim()),
                                    instruction_summary: summary,
                                });
                            }
                        }
                    }
                }
                _ => {
                    instructions.push(InstructionInfo {
                        program_title: "Unsupported encoding".to_string(),
                        instruction_summary: vec![
                            "Switch to JsonParsed encoding to view instruction details."
                                .to_string(),
                        ],
                    });
                }
            }

            let logs = option_serializer_to_vec(&meta.log_messages);
            let compute_units = option_serializer_to_option(&meta.compute_units_consumed);
            let return_data = option_serializer_to_option(&meta.return_data).map(|data| {
                ReturnDataInfo {
                    program_id: data.program_id,
                    data_base64: data.data.0,
                }
            });

            let details = TransactionDetails {
                signature: signature_str.to_string(),
                slot,
                block_time,
                status,
                fee,
                instructions,
                accounts,
                logs,
                compute_units,
                return_data,
            };

            (Some(details), None)
        }
        Err(e) => {
            info!("Failed to fetch transaction: {}", e);
            (None, Some(e.to_string()))
        }
    };
    let mut instructions_scroll: u16 = 0;
    let mut accounts_scroll: u16 = 0;
    let mut instructions_area: Option<Rect> = None;
    let mut accounts_area: Option<Rect> = None;
    let mut logs_area: Option<Rect> = None;
    let mut instructions_content_len: usize = 0;
    let mut accounts_content_len: usize = 0;
    let mut logs_content_len: usize = 0;
    let mut instructions_view_height: usize = 0;
    let mut accounts_view_height: usize = 0;
    let mut logs_view_height: usize = 0;
    let mut logs_scroll: u16 = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Header
            render_banner(
                f,
                chunks[0],
                "Solify Transaction Inspector",
                Some("[q: quit | r: refresh]"),
            );

            // Transaction info
            if let Some(ref details) = tx_details {
                // Overview section
                let sig_short = if details.signature.len() > 20 {
                    format!("{}...{}", &details.signature[..10], &details.signature[details.signature.len()-8..])
                } else {
                    details.signature.clone()
                };

                let fee_sol = details.fee as f64 / 1_000_000_000.0;
                
                let mut overview = vec![
                    format!("Signature: {}", sig_short),
                    format!("Status: {}", details.status),
                    format!("Block: {}", details.slot),
                    format!("Timestamp: {}", details.block_time),
                    format!("Fee: {:.9} SOL", fee_sol),
                    format!("Instructions: {}", details.instructions.len()),
                    format!("Accounts: {}", details.accounts.len()),
                ];
                if let Some(cu) = details.compute_units {
                    overview.push(format!("Compute Units: {}", cu));
                }

                render_info_box(f, chunks[1], "", overview);

                // Split main area into sections
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(chunks[2]);

                let left_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(main_chunks[0]);

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(main_chunks[1]);

                let mut instruction_lines = Vec::new();
                instruction_lines.push(String::new());
                if details.instructions.is_empty() {
                    instruction_lines.push("No instructions parsed".to_string());
                } else {
                    for info in &details.instructions {
                        instruction_lines.push(info.program_title.clone());
                        instruction_lines.extend(info.instruction_summary.clone());
                        instruction_lines.push(String::new());
                    }
                }
                instructions_content_len = instruction_lines.len();
                instructions_view_height = left_chunks[0].height.saturating_sub(2) as usize;
                clamp_scroll(&mut instructions_scroll, instructions_content_len, instructions_view_height);
                render_scrollable_info_box(
                    f,
                    left_chunks[0],
                    "Instructions",
                    instruction_lines,
                    instructions_scroll,
                );
                instructions_area = Some(left_chunks[0]);

                let log_lines = if details.logs.is_empty() {
                    vec![
                        "LOGS".to_string(),
                        String::new(),
                        "No logs available".to_string(),
                        "".to_string(),
                        "Logs may not be available or were not stored.".to_string(),
                    ]
                } else {
                    let mut lines = vec![String::new()];
                    lines.extend(details.logs.iter().cloned());
                    lines
                };
                logs_content_len = log_lines.len();
                logs_view_height = left_chunks[1].height.saturating_sub(2) as usize;
                clamp_scroll(&mut logs_scroll, logs_content_len, logs_view_height);
                render_scrollable_info_box(
                    f,
                    left_chunks[1],
                    "Logs",
                    log_lines,
                    logs_scroll,
                );
                logs_area = Some(left_chunks[1]);

                let mut account_lines = Vec::new();
                account_lines.push(String::new());
                if details.accounts.is_empty() {
                    account_lines.push("No account metadata available".to_string());
                } else {
                    for (idx, account) in details.accounts.iter().enumerate() {
                        account_lines.push(format!("{}. {}", idx + 1, account.pubkey));

                        let mut flags = Vec::new();
                        if account.is_signer {
                            flags.push("Signer");
                        }
                        if account.is_writable {
                            flags.push("Writable");
                        }
                        if !flags.is_empty() {
                            account_lines.push(format!("   {}", flags.join(" | ")));
                        }
                        if let Some(source) = &account.source {
                            account_lines.push(format!("   Source: {}", source));
                        }

                        let balance_change = account.post_balance as i64 - account.pre_balance as i64;
                        if balance_change != 0 {
                            account_lines.push(format!(
                                "   ΔBalance: {}{:.9} SOL",
                                if balance_change > 0 { "+" } else { "" },
                                balance_change as f64 / 1_000_000_000.0
                            ));
                        }
                        account_lines.push(format!(
                            "   Balance: {:.9} SOL",
                            account.post_balance as f64 / 1_000_000_000.0
                        ));

                        for token_line in &account.token_balances {
                            account_lines.push(format!("   {}", token_line));
                        }

                        account_lines.push(String::new());
                    }
                }
                accounts_content_len = account_lines.len();
                accounts_view_height = right_chunks[0].height.saturating_sub(2) as usize;
                clamp_scroll(&mut accounts_scroll, accounts_content_len, accounts_view_height);
                render_scrollable_info_box(
                    f,
                    right_chunks[0],
                    "Accounts",
                    account_lines,
                    accounts_scroll,
                );
                accounts_area = Some(right_chunks[0]);

                let mut return_lines = Vec::new();
                if let Some(return_data) = &details.return_data {
                    return_lines.push("RETURN DATA".to_string());
                    return_lines.push(String::new());
                    return_lines.push(format!("Program: {}", return_data.program_id));
                    let preview_len = return_data
                        .data_base64
                        .len()
                        .min(80);
                    return_lines.push(format!(
                        "Data (base64, {} chars): {}{}",
                        return_data.data_base64.len(),
                        &return_data.data_base64[..preview_len],
                        if return_data.data_base64.len() > preview_len {
                            "..."
                        } else {
                            ""
                        }
                    ));
                } else {
                    return_lines.push("Return data not present".to_string());
                    return_lines.push(String::new());
                    return_lines.push("Tip: Run with --detailed to view more RPC fields.".to_string());
                }

                render_info_box(f, right_chunks[1], "Additional Info", return_lines);

                let status_msg =
                    "Mouse wheel: scroll instructions/accounts/logs | r: refresh | q: quit";
                render_status(f, chunks[3], status_msg, false);
            } else if let Some(ref err) = error_msg {
                instructions_area = None;
                accounts_area = None;
                logs_area = None;
                instructions_content_len = 0;
                accounts_content_len = 0;
                logs_content_len = 0;
                instructions_view_height = 0;
                accounts_view_height = 0;
                logs_view_height = 0;
                instructions_scroll = 0;
                accounts_scroll = 0;
                logs_scroll = 0;
                render_info_box(
                    f,
                    chunks[1],
                    "Error",
                    vec![
                        "Failed to fetch transaction".to_string(),
                        format!("Error: {}", err),
                        "".to_string(),
                        "Note: This may be because:".to_string(),
                        "- The transaction doesn't exist".to_string(),
                        "- The RPC node doesn't have this transaction".to_string(),
                        "- The signature is invalid".to_string(),
                    ],
                );

                render_status(f, chunks[3], "Transaction not found", true);
            } else {
                instructions_area = None;
                accounts_area = None;
                logs_area = None;
                instructions_content_len = 0;
                accounts_content_len = 0;
                logs_content_len = 0;
                instructions_view_height = 0;
                accounts_view_height = 0;
                logs_view_height = 0;
                instructions_scroll = 0;
                accounts_scroll = 0;
                logs_scroll = 0;
                render_info_box(
                    f,
                    chunks[1],
                    "Loading",
                    vec!["Fetching transaction data...".to_string()],
                );

                render_status(f, chunks[3], "Loading...", false);
            }
        })?;

        match event_handler.next()? {
            AppEvent::Quit => break,
            AppEvent::Char('r') | AppEvent::Char('R') => {
                info!("Refreshing transaction data...");
                // Would re-fetch transaction here
            }
            AppEvent::MouseScroll { up, column, row } => {
                let mut handled = false;
                if let Some(area) = instructions_area {
                    if point_in_rect(area, column, row) {
                        adjust_scroll(
                            &mut instructions_scroll,
                            up,
                            instructions_content_len,
                            instructions_view_height,
                        );
                        handled = true;
                    }
                }
                if !handled {
                    if let Some(area) = accounts_area {
                        if point_in_rect(area, column, row) {
                            adjust_scroll(
                                &mut accounts_scroll,
                                up,
                                accounts_content_len,
                                accounts_view_height,
                            );
                            handled = true;
                        }
                    }
                }
                if !handled {
                    if let Some(area) = logs_area {
                        if point_in_rect(area, column, row) {
                            adjust_scroll(
                                &mut logs_scroll,
                                up,
                                logs_content_len,
                                logs_view_height,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    restore_terminal(terminal)?;

    // Print summary to console
    if let Some(_) = tx_details {
        println!("\n✓ Transaction inspection complete");
        println!("  Signature: {}", signature_str);
    } else if let Some(err) = error_msg {
        println!("\n✗ Failed to inspect transaction");
        println!("  Error: {}", err);
    }

    Ok(())
}

fn point_in_rect(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn compute_max_scroll(content_len: usize, view_height: usize) -> u16 {
    if view_height == 0 || content_len <= view_height {
        0
    } else {
        ((content_len - view_height).min(u16::MAX as usize)) as u16
    }
}

fn clamp_scroll(scroll: &mut u16, content_len: usize, view_height: usize) {
    let max_scroll = compute_max_scroll(content_len, view_height);
    if max_scroll == 0 {
        *scroll = 0;
    } else if *scroll > max_scroll {
        *scroll = max_scroll;
    }
}

fn adjust_scroll(
    scroll: &mut u16,
    up: bool,
    content_len: usize,
    view_height: usize,
) {
    let max_scroll = compute_max_scroll(content_len, view_height);
    if max_scroll == 0 {
        *scroll = 0;
        return;
    }
    if up {
        *scroll = scroll.saturating_sub(1);
    } else if *scroll < max_scroll {
        *scroll = (*scroll + 1).min(max_scroll);
    }
}

