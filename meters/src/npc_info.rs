pub const ACTIVE_NPC: NpcInfo = NpcInfo { active: true };
pub const INACTIVE_NPC: NpcInfo = NpcInfo { active: false };

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NpcInfo {
    pub active: bool,
}
