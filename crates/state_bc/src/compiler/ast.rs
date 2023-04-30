use crate::EnemySpawnInfo;

#[derive(Debug)]
pub enum ToplevelElement {
    EnumDecl(EnumDecl),
    StatesBlock(StatesBlock),
    SpawnBlock(SpawnBlock),
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
