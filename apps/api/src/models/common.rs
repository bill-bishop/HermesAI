
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamFrame {
    pub t: String,  // stdout|stderr|event
    pub seq: u64,
    pub d: String,
}
