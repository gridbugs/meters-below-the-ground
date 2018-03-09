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
    Compass,
    Metabol,
    Blink,
    Push,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActiveMeterType {
    Gun,
    RailGun,
    Medkit,
    Metabol,
    Push,
    Blink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveMeterType {
    Health,
    Kevlar,
    Stamina,
    Compass,
}

impl ActiveMeterType {
    pub fn typ(self) -> MeterType {
        self.into()
    }
}

impl PassiveMeterType {
    pub fn typ(self) -> MeterType {
        self.into()
    }
}

impl From<ActiveMeterType> for MeterType {
    fn from(typ: ActiveMeterType) -> Self {
        match typ {
            ActiveMeterType::Gun => MeterType::Gun,
            ActiveMeterType::RailGun => MeterType::RailGun,
            ActiveMeterType::Medkit => MeterType::Medkit,
            ActiveMeterType::Blink => MeterType::Blink,
            ActiveMeterType::Metabol => MeterType::Metabol,
            ActiveMeterType::Push => MeterType::Push,
        }
    }
}

impl From<PassiveMeterType> for MeterType {
    fn from(typ: PassiveMeterType) -> Self {
        match typ {
            PassiveMeterType::Health => MeterType::Health,
            PassiveMeterType::Kevlar => MeterType::Kevlar,
            PassiveMeterType::Stamina => MeterType::Stamina,
            PassiveMeterType::Compass => MeterType::Compass,
        }
    }
}

impl MeterType {
    pub fn from_component_type(component_type: ComponentType) -> Option<Self> {
        match component_type {
            ComponentType::GunMeter => Some(MeterType::Gun),
            ComponentType::MedkitMeter => Some(MeterType::Medkit),
            ComponentType::BlinkMeter => Some(MeterType::Blink),
            ComponentType::MetabolMeter => Some(MeterType::Metabol),
            ComponentType::PushMeter => Some(MeterType::Push),
            ComponentType::StaminaMeter => Some(MeterType::Stamina),
            ComponentType::RailGunMeter => Some(MeterType::RailGun),
            ComponentType::HealthMeter => Some(MeterType::Health),
            ComponentType::KevlarMeter => Some(MeterType::Kevlar),
            ComponentType::CompassMeter => Some(MeterType::Compass),
            _ => None,
        }
    }
    pub fn active_or_passive(self) -> ActiveOrPassive {
        match self {
            MeterType::Gun => ActiveOrPassive::Active(ActiveMeterType::Gun),
            MeterType::RailGun => ActiveOrPassive::Active(ActiveMeterType::RailGun),
            MeterType::Medkit => ActiveOrPassive::Active(ActiveMeterType::Medkit),
            MeterType::Blink => ActiveOrPassive::Active(ActiveMeterType::Blink),
            MeterType::Metabol => ActiveOrPassive::Active(ActiveMeterType::Metabol),
            MeterType::Push => ActiveOrPassive::Active(ActiveMeterType::Push),
            MeterType::Stamina => ActiveOrPassive::Passive(PassiveMeterType::Stamina),
            MeterType::Health => ActiveOrPassive::Passive(PassiveMeterType::Health),
            MeterType::Kevlar => ActiveOrPassive::Passive(PassiveMeterType::Kevlar),
            MeterType::Compass => ActiveOrPassive::Passive(PassiveMeterType::Compass),
        }
    }
    pub fn active(self) -> Option<ActiveMeterType> {
        match self.active_or_passive() {
            ActiveOrPassive::Active(typ) => Some(typ),
            ActiveOrPassive::Passive(_) => None,
        }
    }
    pub fn passive(self) -> Option<PassiveMeterType> {
        match self.active_or_passive() {
            ActiveOrPassive::Active(_) => None,
            ActiveOrPassive::Passive(typ) => Some(typ),
        }
    }

    pub fn player_max(self) -> i32 {
        match self {
            MeterType::Gun => 8,
            MeterType::RailGun => 8,
            MeterType::Medkit => 6,
            MeterType::Blink => 8,
            MeterType::Metabol => 4,
            MeterType::Push => 4,
            MeterType::Stamina => 6,
            MeterType::Health => 10,
            MeterType::Kevlar => 10,
            MeterType::Compass => 100,
        }
    }
    pub fn player_component_value(self) -> ComponentValue {
        let max = self.player_max();
        let initial = max;
        match self {
            MeterType::Gun => ComponentValue::GunMeter(Meter::new(initial, max)),
            MeterType::RailGun => ComponentValue::RailGunMeter(Meter::new(initial, max)),
            MeterType::Medkit => ComponentValue::MedkitMeter(Meter::new(initial, max)),
            MeterType::Blink => ComponentValue::BlinkMeter(Meter::new(initial, max)),
            MeterType::Metabol => ComponentValue::MetabolMeter(Meter::new(initial, max)),
            MeterType::Push => ComponentValue::PushMeter(Meter::new(initial, max)),
            MeterType::Stamina => ComponentValue::StaminaMeter(Meter::new(initial, max)),
            MeterType::Health => ComponentValue::HealthMeter(Meter::full(max)),
            MeterType::Kevlar => ComponentValue::KevlarMeter(Meter::new(initial, max)),
            MeterType::Compass => ComponentValue::CompassMeter(Meter::new(initial, max)),
        }
    }
    pub fn is_active(self) -> bool {
        match self {
            MeterType::Gun => true,
            MeterType::RailGun => true,
            MeterType::Medkit => true,
            MeterType::Blink => true,
            MeterType::Metabol => true,
            MeterType::Push => true,
            MeterType::Stamina => false,
            MeterType::Health => false,
            MeterType::Kevlar => false,
            MeterType::Compass => false,
        }
    }
    pub fn insert(self, id: EntityId, meter: Meter) -> EntityChange {
        match self {
            MeterType::Gun => insert::gun_meter(id, meter),
            MeterType::RailGun => insert::rail_gun_meter(id, meter),
            MeterType::Medkit => insert::medkit_meter(id, meter),
            MeterType::Blink => insert::blink_meter(id, meter),
            MeterType::Metabol => insert::metabol_meter(id, meter),
            MeterType::Push => insert::push_meter(id, meter),
            MeterType::Health => insert::health_meter(id, meter),
            MeterType::Stamina => insert::stamina_meter(id, meter),
            MeterType::Kevlar => insert::kevlar_meter(id, meter),
            MeterType::Compass => insert::compass_meter(id, meter),
        }
    }
    pub fn periodic_change(self) -> Option<PeriodicChange> {
        match self {
            MeterType::Gun => None,
            MeterType::RailGun => None,
            MeterType::Blink => Some(PeriodicChange {
                turns: 0,
                change: 1,
            }),
            MeterType::Medkit => Some(PeriodicChange {
                turns: 0,
                change: 1,
            }),
            MeterType::Health => None,
            MeterType::Stamina => Some(PeriodicChange {
                turns: 0,
                change: 1,
            }),
            MeterType::Kevlar => None,
            MeterType::Metabol => None,
            MeterType::Push => None,
            MeterType::Compass => None,
        }
    }
}

impl From<MeterType> for ComponentType {
    fn from(typ: MeterType) -> Self {
        match typ {
            MeterType::Gun => ComponentType::GunMeter,
            MeterType::RailGun => ComponentType::RailGunMeter,
            MeterType::Medkit => ComponentType::MedkitMeter,
            MeterType::Blink => ComponentType::BlinkMeter,
            MeterType::Metabol => ComponentType::MetabolMeter,
            MeterType::Push => ComponentType::PushMeter,
            MeterType::Health => ComponentType::HealthMeter,
            MeterType::Stamina => ComponentType::StaminaMeter,
            MeterType::Kevlar => ComponentType::KevlarMeter,
            MeterType::Compass => ComponentType::CompassMeter,
        }
    }
}

pub const ALL_METER_TYPES: &[MeterType] = &[
    MeterType::Gun,
    MeterType::RailGun,
    MeterType::Medkit,
    MeterType::Metabol,
    MeterType::Push,
    MeterType::Health,
    MeterType::Stamina,
    MeterType::Kevlar,
    MeterType::Compass,
    MeterType::Blink,
];

pub struct PeriodicChange {
    pub turns: u32,
    pub change: i32,
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
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
        Meter {
            max,
            value: ::std::cmp::min(value, max),
        }
    }
    pub fn from_component_ref(component: ComponentRef) -> Option<Self> {
        match component {
            ComponentRef::HealthMeter(meter) => Some(*meter),
            ComponentRef::StaminaMeter(meter) => Some(*meter),
            ComponentRef::GunMeter(meter) => Some(*meter),
            ComponentRef::RailGunMeter(meter) => Some(*meter),
            ComponentRef::KevlarMeter(meter) => Some(*meter),
            ComponentRef::MedkitMeter(meter) => Some(*meter),
            ComponentRef::BlinkMeter(meter) => Some(*meter),
            ComponentRef::MetabolMeter(meter) => Some(*meter),
            ComponentRef::PushMeter(meter) => Some(*meter),
            ComponentRef::CompassMeter(meter) => Some(*meter),
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
