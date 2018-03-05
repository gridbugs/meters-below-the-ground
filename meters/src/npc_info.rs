#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct NpcInfo {
    pub active: bool,
    pub boss: bool,
}
