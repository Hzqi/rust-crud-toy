use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceCreateParam {
    pub name: String,
    pub description: String,
    pub creator: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceUpdateParam {
    pub name: String,
    pub description: String,
}
