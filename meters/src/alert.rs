#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alert {
    NoStamina,
    NoAmmo,
    NoMedkit,
    ArmourBlock,
    WalkIntoWall,
}
