use std::fmt::Write;
use prototty::*;
use prototty_common::*;
use meters::meter::*;
use meters::goal::*;

fn meter_text_info(typ: MeterType) -> TextInfo {
    let colour = match typ {
        MeterType::Gun => colours::BRIGHT_BLUE,
        MeterType::Medkit => colours::CYAN,
        MeterType::Health => colours::BRIGHT_RED,
        MeterType::Kevlar => colours::BRIGHT_YELLOW,
    };
    TextInfo {
        foreground_colour: Some(colour),
        ..Default::default()
    }
}

fn goal_meter_text_info(typ: GoalMeterType) -> TextInfo {
    let colour = match typ {
        GoalMeterType::BossHealth => colours::BRIGHT_MAGENTA,
    };
    TextInfo {
        foreground_colour: Some(colour),
        ..Default::default()
    }
}

pub struct MeterView {
    name_padding: usize,
    meter_width: usize,
    scratch: String,
}

impl MeterView {
    pub fn new(name_padding: usize, meter_width: usize) -> Self {
        Self {
            name_padding,
            meter_width,
            scratch: String::new(),
        }
    }
    fn write_active_name(&mut self, typ: ActiveMeterType, identifier: char, is_selected: bool) {
        let seperator = if is_selected { "*" } else { " " };

        write!(self.scratch, "{}){}", identifier, seperator).unwrap();
        match typ {
            ActiveMeterType::Gun => {
                write!(self.scratch, "{:1$}", "Gun", self.name_padding).unwrap()
            }
            ActiveMeterType::Medkit => {
                write!(self.scratch, "{:1$}", "Medkit", self.name_padding).unwrap()
            }
        }
    }
    fn write_passive_name(&mut self, typ: PassiveMeterType) {
        write!(self.scratch, "   ").unwrap();
        match typ {
            PassiveMeterType::Health => {
                write!(self.scratch, "{:1$}", "Health", self.name_padding).unwrap()
            }
            PassiveMeterType::Kevlar => {
                write!(self.scratch, "{:1$}", "Kevlar", self.name_padding).unwrap()
            }
        }
    }
    fn write_goal_name(&mut self, typ: GoalMeterType) {
        write!(self.scratch, "   ").unwrap();
        match typ {
            GoalMeterType::BossHealth => {
                write!(self.scratch, "{:1$}", "Boss", self.name_padding).unwrap()
            }
        }
    }
    fn write_meter(&mut self, meter: Meter) {
        let value = ::std::cmp::max(meter.value, 0) as usize;
        let max = ::std::cmp::max(meter.max, 0) as usize;
        let filled_meter_width = (self.meter_width * value) / max;
        let remaining_meter_width = self.meter_width - filled_meter_width;
        for _ in 0..filled_meter_width {
            self.scratch.push('█');
        }
        for _ in 0..remaining_meter_width {
            self.scratch.push('░')
        }

        write!(self.scratch, " {}/{}", meter.value, meter.max).unwrap();
    }
}

impl View<ActiveMeterInfo> for MeterView {
    fn view<G: ViewGrid>(
        &mut self,
        info: &ActiveMeterInfo,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.scratch.clear();
        self.write_active_name(info.typ, info.identifier.to_char(), info.is_selected);
        self.write_meter(info.meter);
        let info = meter_text_info(info.typ.typ());
        TextInfoStringView.view(&(info, &self.scratch), offset, depth, grid);
    }
}

impl View<PassiveMeterInfo> for MeterView {
    fn view<G: ViewGrid>(
        &mut self,
        info: &PassiveMeterInfo,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.scratch.clear();
        self.write_passive_name(info.typ);
        self.write_meter(info.meter);
        let info = meter_text_info(info.typ.typ());
        TextInfoStringView.view(&(info, &self.scratch), offset, depth, grid);
    }
}

impl View<GoalMeterInfo> for MeterView {
    fn view<G: ViewGrid>(&mut self, info: &GoalMeterInfo, offset: Coord, depth: i32, grid: &mut G) {
        self.scratch.clear();
        self.write_goal_name(info.typ);
        self.write_meter(info.meter);
        let info = goal_meter_text_info(info.typ);
        TextInfoStringView.view(&(info, &self.scratch), offset, depth, grid);
    }
}

impl View<(&'static str, Meter)> for MeterView {
    fn view<G: ViewGrid>(
        &mut self,
        &(title, meter): &(&'static str, Meter),
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.scratch.clear();
        write!(self.scratch, "{} ", title).unwrap();
        self.write_meter(meter);
        StringView.view(&self.scratch, offset, depth, grid);
    }
}
