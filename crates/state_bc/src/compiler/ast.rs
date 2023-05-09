use crate::EnemySpawnInfo;

#[derive(Debug)]
pub enum ToplevelElement {
    EnumDecl(EnumDecl),
    StatesBlock(StatesBlock),
    SpawnBlock(SpawnBlock),
    FunctionBlock(FunctionBlock),
}

#[derive(Debug)]
pub struct EnumDecl {
    pub names: Vec<String>,
}

#[derive(Debug)]
pub enum StatesBlockElement {
    Label(String),
    State {
        id: String,
        directional: bool,
        ticks: i32,
        think: String,
        action: String,
        next: String,
    },
}

#[derive(Debug)]
pub struct StatesBlock {
    pub name: String,
    pub elements: Vec<StatesBlockElement>,
}

#[derive(Debug)]
pub struct SpawnBlock {
    pub name: String,
    pub infos: Vec<EnemySpawnInfo>,
}

#[derive(Debug)]
pub enum FunctionBlockElement {
    Label(String),
    // LoadI32 { addr: u8 },
    LoadiI32 { value: i32 },
    LoadiU8Enum { name: String },
    LoadiU8 { value: u8 },
    // StoreI32 { addr: u8 },
    Trap,
    Add,
    Ceq,
    Not,
    Jrc { label: String },
    FunctionCall,
    Stop,
}
#[derive(Debug)]
pub struct FunctionBlock {
    pub name: String,
    pub elements: Vec<FunctionBlockElement>,
}
