use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlData {
    pub name: String,
    pub version: String,
    pub instructions: Vec<IdlInstruction>,
    #[serde(default)]
    pub accounts: Vec<IdlAccount>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub errors: Vec<IdlError>,
    #[serde(default)]
    pub constants: Vec<IdlConstant>,
    #[serde(default)]
    pub events: Vec<IdlEvent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlInstruction {
    pub name: String,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
    pub docs: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlAccountItem {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
    pub docs: Vec<String>,
    pub pda: Option<IdlPda>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlPda {
    pub seeds: Vec<IdlSeed>,
    #[serde(default)]
    pub program: String,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlSeed {
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub value: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlAccount {
    pub name: String,
    pub fields: Vec<IdlField>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlField {
    pub name: String,
    pub field_type: String, 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IdlTypeDef {
    pub name: String,
    pub kind: String,
    pub fields: Vec<String>,
}