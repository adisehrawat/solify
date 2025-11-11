use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIdl {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub metadata: IdlMetadata,
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub accounts: Vec<AccountDef>,
    #[serde(default)]
    pub types: Vec<TypeDef>,
    #[serde(default)]
    pub errors: Vec<ErrorDef>,
    #[serde(default)]
    pub constants: Vec<ConstantDef>,
    #[serde(default)]
    pub events: Vec<EventDef>,
}



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdlMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub spec: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
    pub accounts: Vec<AccountInfo>,
    pub args: Vec<ArgumentDef>,
    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub name: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub signer: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub pda: Option<PdaConfig>,
    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaConfig {
    pub seeds: Vec<PdaSeed>,
    #[serde(default)]
    pub program: Option<PdaProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaProgram {
    pub kind: String,
    #[serde(default)]
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaSeed {
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentDef {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    Simple(String),
    Vec {
        vec: Box<IdlType>
    },
    Option {
        option: Box<IdlType>
    },
    Array {
        array: (Box<IdlType>, usize)
    },
    Defined {
        defined: DefinedType
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DefinedType {
    Simple(String),
    Generic {
        name: String,
        #[serde(default)]
        generics: Vec<IdlType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDef {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: String,
    #[serde(rename = "type")]
    pub constant_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDef {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_kind: TypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TypeKind {
    #[serde(rename = "struct")]
    Struct { 
        fields: Vec<FieldDef> 
    },
    #[serde(rename = "enum")]
    Enum { 
        variants: Vec<EnumVariant> 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    #[serde(default)]
    pub fields: Option<Vec<FieldDef>>,
}
