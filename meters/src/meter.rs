use entity_store::*;
use input::ActiveMeterIdentifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActiveMeterType {
    GunAmmo,
}

impl ActiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::GunAmmoMeter => Some(ActiveMeterType::GunAmmo),
            _ => None,
        }
    }
}

impl From<ActiveMeterType> for ComponentType {
    fn from(typ: ActiveMeterType) -> Self {
        match typ {
            ActiveMeterType::GunAmmo => ComponentType::GunAmmoMeter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveMeterType {
    Health,
}

impl PassiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::HealthMeter => Some(PassiveMeterType::Health),
            _ => None,
        }
    }
}

impl From<PassiveMeterType> for ComponentType {
    fn from(typ: PassiveMeterType) -> Self {
        match typ {
            PassiveMeterType::Health => ComponentType::HealthMeter,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Meter {
    pub max: i32,
    pub value: i32,
}

impl Meter {
    pub fn full(max: i32) -> Self {
        Meter { max, value: max }
    }
    pub fn from_component_ref(component: ComponentRef) -> Option<Self> {
        match component {
            ComponentRef::HealthMeter(meter) => Some(*meter),
            ComponentRef::GunAmmoMeter(meter) => Some(*meter),
            _ => None,
        }
    }
    pub fn from_entity_store<T: Into<ComponentType>>(
        id: EntityId,
        entity_store: &EntityStore,
        typ: T,
    ) -> Option<Self> {
        let component_type = typ.into();
        entity_store
            .get(id, component_type)
            .and_then(Self::from_component_ref)
    }
}

#[derive(Debug, Clone)]
pub struct ActiveMeterInfo {
    pub identifier: ActiveMeterIdentifier,
    pub is_selected: bool,
    pub typ: ActiveMeterType,
    pub meter: Meter,
}

#[derive(Debug, Clone)]
pub struct PassiveMeterInfo {
    pub typ: PassiveMeterType,
    pub meter: Meter,
}
