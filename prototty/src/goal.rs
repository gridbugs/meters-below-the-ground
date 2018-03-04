use std::fmt::Write;
use prototty::*;
use prototty_common::*;
use meters::goal::*;

pub struct GoalView {
    scratch: String,
}

impl GoalView {
    pub fn new() -> Self {
        Self {
            scratch: String::new(),
        }
    }
}

impl View<GoalType> for GoalView {
    fn view<G: ViewGrid>(&mut self, &goal: &GoalType, offset: Coord, depth: i32, grid: &mut G) {
        self.scratch.clear();
        match goal {
            GoalType::Escape => write!(self.scratch, "Escape!").unwrap(),
        }
        StringView.view(&self.scratch, offset, depth, grid);
    }
}
