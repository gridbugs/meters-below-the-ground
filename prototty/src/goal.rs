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

impl View<Goal> for GoalView {
    fn view<G: ViewGrid>(&mut self, &goal: &Goal, offset: Coord, depth: i32, grid: &mut G) {
        self.scratch.clear();
        match goal {
            Goal::Escape => write!(self.scratch, "Escape!").unwrap(),
        }
        StringView.view(&self.scratch, offset, depth, grid);
    }
}
