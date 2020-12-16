use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ModConfiguration {
    pub id: String,
    pub display_name: String,
}
