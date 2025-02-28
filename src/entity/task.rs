use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,

    pub name: String,

    pub description: String,

    pub test_path: String
}
