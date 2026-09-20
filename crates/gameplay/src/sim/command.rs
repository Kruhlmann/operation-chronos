use std::collections::BTreeMap;

use data::math::FixedVec2;

pub type Tick = u64;
pub const TICK_DELAY_BEFORE_COMMAND_TAKES_EFFECT: u64 = 2;

#[derive(Clone, Debug)]
pub enum PlayerCommand {
    Move {
        units: Vec<hecs::Entity>,
        target: FixedVec2,
    },
}

#[derive(Default, Debug)]
pub struct CommandLog {
    pub history: BTreeMap<Tick, Vec<PlayerCommand>>,
}

impl CommandLog {
    pub fn record(&mut self, tick: Tick, command: PlayerCommand) {
        self.history.entry(tick).or_default().push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = (Tick, &PlayerCommand)> {
        self.history
            .iter()
            .flat_map(|(t, cs)| cs.iter().map(move |c| (*t, c)))
    }
}
