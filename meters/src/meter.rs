#[derive(Debug, Clone, Copy)]
pub enum MeterType {
    Health,
    GunAmmo,
}

#[derive(Debug, Clone)]
pub struct Meter {
    pub identifier: char,
    pub typ: MeterType,
    pub max: u32,
    pub value: u32,
}
