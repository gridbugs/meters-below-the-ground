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

impl View<Option<(GoalType, bool)>> for GoalView {
    fn view<G: ViewGrid>(
        &mut self,
        &goal: &Option<(GoalType, bool)>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        if let Some(&(goal, complete)) = goal.as_ref() {
            self.scratch.clear();
            match goal {
                GoalType::Escape => write!(self.scratch, "Escape!").unwrap(),
                GoalType::KillBoss => write!(self.scratch, "Kill the boss!").unwrap(),
                GoalType::KillEggs => {
                    write!(self.scratch, "Kill the eggs before they hatch!").unwrap()
                }
            }
            if complete {
                write!(self.scratch, " Complete!").unwrap();
            }
            StringView.view(&self.scratch, offset, depth, grid);
        }
    }
}
