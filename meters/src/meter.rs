use entity_store::*;
use input::ActiveMeterIdentifier;

pub enum ActiveOrPassive {
    Active(ActiveMeterType),
    Passive(PassiveMeterType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MeterType {
    Gun,
    RailGun,
    Medkit,
    Stamina,
    Health,
    Kevlar,
}

impl MeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::GunMeter => Some(MeterType::Gun),
            ComponentType::MedkitMeter => Some(MeterType::Medkit),
            ComponentType::StaminaMeter => Some(MeterType::Stamina),
            ComponentType::RailGunMeter => Some(MeterType::RailGun),
            ComponentType::HealthMeter => Some(MeterType::Health),
            ComponentType::KevlarMeter => Some(MeterType::Kevlar),
            _ => None,
        }
    }
    pub fn active_or_passive(self) -> ActiveOrPassive {
        match self {
            MeterType::Gun => ActiveOrPassive::Active(ActiveMeterType::Gun),
            MeterType::RailGun => ActiveOrPassive::Active(ActiveMeterType::RailGun),
            MeterType::Medkit => ActiveOrPassive::Active(ActiveMeterType::Medkit),
            MeterType::Stamina => ActiveOrPassive::Passive(PassiveMeterType::Stamina),
            MeterType::Health => ActiveOrPassive::Passive(PassiveMeterType::Health),
            MeterType::Kevlar => ActiveOrPassive::Passive(PassiveMeterType::Kevlar),
        }
    }
    pub fn player_max(self) -> i32 {
        match self {
            MeterType::Gun => 8,
            MeterType::RailGun => 4,
            MeterType::Medkit => 8,
            MeterType::Stamina => 4,
            MeterType::Health => 10,
            MeterType::Kevlar => 10,
        }
    }
    pub fn player_component_value(self) -> ComponentValue {
        let max = self.player_max();
        let initial = max / 2;
        match self {
            MeterType::Gun => ComponentValue::GunMeter(Meter::new(initial, max)),
            MeterType::RailGun => ComponentValue::RailGunMeter(Meter::new(initial, max)),
            MeterType::Medkit => ComponentValue::MedkitMeter(Meter::new(initial, max)),
            MeterType::Stamina => ComponentValue::StaminaMeter(Meter::new(initial, max)),
            MeterType::Health => ComponentValue::HealthMeter(Meter::full(max)),
            MeterType::Kevlar => ComponentValue::KevlarMeter(Meter::new(initial, max)),
        }
    }
    pub fn is_active(self) -> bool {
        match self {
            MeterType::Gun => true,
            MeterType::RailGun => true,
            MeterType::Medkit => true,
            MeterType::Stamina => false,
            MeterType::Health => false,
            MeterType::Kevlar => false,
        }
    }
}

impl From<MeterType> for ComponentType {
    fn from(typ: MeterType) -> Self {
        match typ {
            MeterType::Gun => ComponentType::GunMeter,
            MeterType::RailGun => ComponentType::RailGunMeter,
            MeterType::Medkit => ComponentType::MedkitMeter,
            MeterType::Health => ComponentType::HealthMeter,
            MeterType::Stamina => ComponentType::StaminaMeter,
            MeterType::Kevlar => ComponentType::KevlarMeter,
        }
    }
}

pub const ALL_METER_TYPES: &[MeterType] = &[
    MeterType::Gun,
    MeterType::RailGun,
    MeterType::Medkit,
    MeterType::Health,
    MeterType::Stamina,
    MeterType::Kevlar,
];

pub struct PeriodicChange {
    pub turns: u32,
    pub change: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActiveMeterType {
    Gun,
    RailGun,
    Medkit,
}

impl ActiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::GunMeter => Some(ActiveMeterType::Gun),
            ComponentType::MedkitMeter => Some(ActiveMeterType::Medkit),
            ComponentType::RailGunMeter => Some(ActiveMeterType::RailGun),
            _ => None,
        }
    }
    pub fn periodic_change(self) -> Option<PeriodicChange> {
        match self {
            ActiveMeterType::Gun => None,
            ActiveMeterType::RailGun => None,
            ActiveMeterType::Medkit => Some(PeriodicChange {
                turns: 3,
                change: 1,
            }),
        }
    }
    pub fn insert(self, id: EntityId, meter: Meter) -> EntityChange {
        match self {
            ActiveMeterType::Gun => insert::gun_meter(id, meter),
            ActiveMeterType::RailGun => insert::rail_gun_meter(id, meter),
            ActiveMeterType::Medkit => insert::medkit_meter(id, meter),
        }
    }
    pub fn typ(self) -> MeterType {
        match self {
            ActiveMeterType::Gun => MeterType::Gun,
            ActiveMeterType::Medkit => MeterType::Medkit,
            ActiveMeterType::RailGun => MeterType::RailGun,
        }
    }
}

impl From<ActiveMeterType> for ComponentType {
    fn from(typ: ActiveMeterType) -> Self {
        match typ {
            ActiveMeterType::Gun => ComponentType::GunMeter,
            ActiveMeterType::Medkit => ComponentType::MedkitMeter,
            ActiveMeterType::RailGun => ComponentType::RailGunMeter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveMeterType {
    Health,
    Kevlar,
    Stamina,
}

impl PassiveMeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::HealthMeter => Some(PassiveMeterType::Health),
            ComponentType::StaminaMeter => Some(PassiveMeterType::Stamina),
            ComponentType::KevlarMeter => Some(PassiveMeterType::Kevlar),
            _ => None,
        }
    }
    pub fn periodic_change(self) -> Option<PeriodicChange> {
        match self {
            PassiveMeterType::Health => None,
            PassiveMeterType::Stamina => Some(PeriodicChange {
                turns: 0,
                change: 1,
            }),
            PassiveMeterType::Kevlar => None,
        }
    }
    pub fn insert(self, id: EntityId, meter: Meter) -> EntityChange {
        match self {
            PassiveMeterType::Health => insert::health_meter(id, meter),
            PassiveMeterType::Stamina => insert::stamina_meter(id, meter),
            PassiveMeterType::Kevlar => insert::kevlar_meter(id, meter),
        }
    }
    pub fn typ(self) -> MeterType {
        match self {
            PassiveMeterType::Health => MeterType::Health,
            PassiveMeterType::Kevlar => MeterType::Kevlar,
            PassiveMeterType::Stamina => MeterType::Stamina,
        }
    }
}

impl From<PassiveMeterType> for ComponentType {
    fn from(typ: PassiveMeterType) -> Self {
        match typ {
            PassiveMeterType::Health => ComponentType::HealthMeter,
            PassiveMeterType::Stamina => ComponentType::StaminaMeter,
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
    pub fn new(value: i32, max: i32) -> Self {
        Meter { max, value }
    }
    pub fn from_component_ref(component: ComponentRef) -> Option<Self> {
        match component {
            ComponentRef::HealthMeter(meter) => Some(*meter),
            ComponentRef::StaminaMeter(meter) => Some(*meter),
            ComponentRef::GunMeter(meter) => Some(*meter),
            ComponentRef::RailGunMeter(meter) => Some(*meter),
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
