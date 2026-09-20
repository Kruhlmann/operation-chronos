use std::collections::BTreeMap;

use super::command::{PlayerCommand, Tick};

#[derive(Default, Debug)]
pub struct PendingCommands(BTreeMap<Tick, Vec<PlayerCommand>>);

impl PendingCommands {
    #[inline]
    pub fn schedule_player_command_for_tick(&mut self, apply_tick: Tick, command: PlayerCommand) {
        self.0.entry(apply_tick).or_default().push(command);
    }

    pub fn drain_player_commands_until_tick(&mut self, up_to: Tick) -> Vec<(Tick, PlayerCommand)> {
        let mut out = Vec::new();
        let ready_keys: Vec<Tick> = self.0.range(..=up_to).map(|(k, _)| *k).collect();
        for k in ready_keys {
            if let Some(v) = self.0.remove(&k) {
                for c in v {
                    out.push((k, c));
                }
            }
        }
        out
    }
}
