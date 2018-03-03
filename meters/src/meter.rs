use entity_store::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MeterType {
    Health,
    GunAmmo,
}

impl MeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::HealthMeter => Some(MeterType::Health),
            ComponentType::GunAmmoMeter => Some(MeterType::GunAmmo),
            _ => None,
        }
    }
}

impl From<MeterType> for ComponentType {
    fn from(meter_type: MeterType) -> Self {
        match meter_type {
            MeterType::Health => ComponentType::HealthMeter,
            MeterType::GunAmmo => ComponentType::GunAmmoMeter,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Meter {
    pub max: u32,
    pub value: u32,
}

impl Meter {
    pub fn full(max: u32) -> Self {
        Meter {
            max,
            value: max,
        }
    }
    pub fn from_component_ref(component: ComponentRef) -> Option<Self> {
        match component {
            ComponentRef::HealthMeter(meter) => Some(*meter),
            ComponentRef::GunAmmoMeter(meter) => Some(*meter),
            _ => None,
        }
    }
    pub fn from_entity_store(id: EntityId, entity_store: &EntityStore, typ: MeterType) -> Option<Self> {
        let component_type = typ.into();
        entity_store.get(id, component_type).and_then(Self::from_component_ref)
    }
}

#[derive(Debug, Clone)]
pub struct MeterInfo {
    pub identifier: char,
    pub typ: MeterType,
    pub meter: Meter,
}