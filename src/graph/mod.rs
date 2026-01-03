// Graph-based pattern matching (placeholder for future implementation)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    // Placeholder - will use petgraph in future
}

impl EntityGraph {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EntityGraph {
    fn default() -> Self {
        Self::new()
    }
}
