#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alert {
    NoStamina,
    NoAmmo,
    NoMedkit,
    NoBlink,
    ArmourBlock,
    WalkIntoWall,
    BlinkIntoNonEmpty,
    NoSuchMeter,
    RailgunWhichDirection,
    BlinkWhichDirection,
    BeaconActive,
}
