#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Pickup {
    Health,
    Ammo,
    RailGunAmmo,
    Kevlar,
}
