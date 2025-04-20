use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resident {
    pub id: u32,
    pub name: String,
    pub wallet: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub resident_id: u32,
    pub amount_copt: u64,
    pub timestamp: DateTime<Utc>,
}

impl Payment {
    pub fn new(resident_id: u32, amount: u64) -> Self {
        Payment {
            resident_id,
            amount_copt: amount,
            timestamp: Utc::now(),
        }
    }
}