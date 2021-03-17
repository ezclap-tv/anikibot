use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldTotal {
    pub count: usize,
    pub amount: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsTotals {
    /// Amount may be None.
    pub follow: FieldTotal,
    // Amount may be None.
    pub subscriber: FieldTotal,
    pub tip: FieldTotal,
    pub host: FieldTotal,
    pub raid: FieldTotal,
    pub cheer: FieldTotal,
    pub merch: FieldTotal,
    pub redemption: FieldTotal,
}

/// Used only for deserialization
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AllStats {
    pub(crate) totals: StatsTotals,
}
