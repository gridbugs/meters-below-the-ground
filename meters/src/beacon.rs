#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeaconStatus {
    Active,
    Inactive,
}
