use meters::goal::*;
use prototty::*;
use std::fmt::Write;

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
                GoalType::ActivateBeacon => {
                    write!(self.scratch, "Activate the emergency beacon!").unwrap()
                }
            }
            if complete {
                write!(self.scratch, " (COMPLETE)").unwrap();
            }
            StringView.view(&self.scratch, offset, depth, grid);
        }
    }
}
