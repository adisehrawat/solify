use anchor_lang::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlData {
    #[max_len(30)]
    pub name: String,
    #[max_len(10)]
    pub version: String,
    #[max_len(3)]
    pub instructions: Vec<IdlInstruction>,
    #[max_len(5)]
    #[serde(default)]
    pub accounts: Vec<IdlAccount>,
    #[max_len(5)]
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[max_len(10)]
    #[serde(default)]
    pub errors: Vec<IdlError>,
    #[max_len(3)]
    #[serde(default)]
    pub constants: Vec<IdlConstant>,
    #[max_len(3)]
    #[serde(default)]
    pub events: Vec<IdlEvent>,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlInstruction {
    #[max_len(30)]
    pub name: String,
    #[max_len(5)]
    pub accounts: Vec<IdlAccountItem>,
    #[max_len(5)]
    pub args: Vec<IdlField>,
    #[max_len(2, 50)]
    pub docs: Vec<String>,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlAccountItem {
    #[max_len(30)]
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
    #[max_len(1, 50)]
    pub docs: Vec<String>,
    pub pda: Option<IdlPda>,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlPda {
    #[max_len(3)]
    pub seeds: Vec<IdlSeed>,
    #[serde(default)]
    #[max_len(30)]
    pub program: Option<String>,
}


#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlSeed {
    #[max_len(20)]
    pub kind: String,
    #[max_len(50)]
    #[serde(default)]
    pub path: String,
    #[max_len(50)]
    #[serde(default)]
    pub value: String,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlAccount {
    #[max_len(30)]
    pub name: String,
    #[max_len(10)]
    pub fields: Vec<IdlField>,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlField {
    #[max_len(30)]
    pub name: String,
    #[max_len(50)]
    pub field_type: String, 
}

#[derive(AnchorSerialize, AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlTypeDef {
    #[max_len(30)]
    pub name: String,
    #[max_len(15)]
    pub kind: String,
    #[max_len(10, 30)]
    pub fields: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlError {
    pub code: u32,
    #[max_len(30)]
    pub name: String,
    #[max_len(50)]
    pub msg: String,
}

#[derive(AnchorSerialize, AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlConstant {
    #[max_len(30)]
    pub name: String,
    #[max_len(30)]
    pub constant_type: String,
    #[max_len(30)]
    pub value: String,
}

#[derive(AnchorSerialize, AnchorDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    InitSpace
)]
pub struct IdlEvent {
    #[max_len(30)]
    pub name: String,
    #[max_len(8)]
    pub discriminator: Vec<u8>,
    #[max_len(5)]
    pub fields: Vec<IdlField>,
}