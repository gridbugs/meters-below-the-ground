use entity_store::*;
use input::ActiveMeterIdentifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MeterType {
    Gun,
    Medkit,
    Health,
    Kevlar,
}

pub struct PeriodicChange {
    pub turns: u32,
    pub change: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActiveMeterType {
    Gun,
    Medkit,
}

impl ActiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::GunMeter => Some(ActiveMeterType::Gun),
            ComponentType::MedkitMeter => Some(ActiveMeterType::Medkit),
            _ => None,
        }
    }
    pub fn periodic_change(self) -> Option<PeriodicChange> {
        match self {
            ActiveMeterType::Gun => None,
            ActiveMeterType::Medkit => Some(PeriodicChange {
                turns: 8,
                change: 1,
            }),
        }
    }
    pub fn insert(self, id: EntityId, meter: Meter) -> EntityChange {
        match self {
            ActiveMeterType::Gun => insert::gun_meter(id, meter),
            ActiveMeterType::Medkit => insert::medkit_meter(id, meter),
        }
    }
    pub fn typ(self) -> MeterType {
        match self {
            ActiveMeterType::Gun => MeterType::Gun,
            ActiveMeterType::Medkit => MeterType::Medkit,
        }
    }
}

impl From<ActiveMeterType> for ComponentType {
    fn from(typ: ActiveMeterType) -> Self {
        match typ {
            ActiveMeterType::Gun => ComponentType::GunMeter,
            ActiveMeterType::Medkit => ComponentType::MedkitMeter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveMeterType {
    Health,
    Kevlar,
}

impl PassiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::HealthMeter => Some(PassiveMeterType::Health),
            ComponentType::KevlarMeter => Some(PassiveMeterType::Kevlar),
            _ => None,
        }
    }
    pub fn periodic_change(self) -> Option<PeriodicChange> {
        match self {
            PassiveMeterType::Health => None,
            PassiveMeterType::Kevlar => None,
        }
    }
    pub fn insert(self, id: EntityId, meter: Meter) -> EntityChange {
        match self {
            PassiveMeterType::Health => insert::health_meter(id, meter),
            PassiveMeterType::Kevlar => insert::kevlar_meter(id, meter),
        }
    }
    pub fn typ(self) -> MeterType {
        match self {
            PassiveMeterType::Health => MeterType::Health,
            PassiveMeterType::Kevlar => MeterType::Kevlar,
        }
    }
}

impl From<PassiveMeterType> for ComponentType {
    fn from(typ: PassiveMeterType) -> Self {
        match typ {
            PassiveMeterType::Health => ComponentType::HealthMeter,
            PassiveMeterType::Kevlar => ComponentType::KevlarMeter,
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
    pub fn empty(max: i32) -> Self {
        Meter { max, value: 0 }
    }
    pub fn from_component_ref(component: ComponentRef) -> Option<Self> {
        match component {
            ComponentRef::HealthMeter(meter) => Some(*meter),
            ComponentRef::GunMeter(meter) => Some(*meter),
            ComponentRef::KevlarMeter(meter) => Some(*meter),
            ComponentRef::MedkitMeter(meter) => Some(*meter),
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
