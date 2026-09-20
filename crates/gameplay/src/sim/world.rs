use core::time::Duration;

use data::constants::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH};
use data::geometry::{Disc, Facing, Footprint, Position};
use data::math::{FixedVec2, Scalar};
use world::entity::{Health, Name, Speed, UnitKind, UnitMoveInstructions, UnitMovementMarker};
use world::{Map, Tile};

use crate::pathfinding::{AStarPathFindingAlgorithm, PathFinder, PathFindingResult, Waypoints};
use crate::sim::collision::resolve_collisions;
use crate::sim::command::{
    CommandLog, PlayerCommand, TICK_DELAY_BEFORE_COMMAND_TAKES_EFFECT, Tick,
};
use crate::sim::markers::tick_markers;
use crate::sim::movement::run_movement;
use crate::sim::scheduler::PendingCommands;

pub const UNIT_PICK_RADIUS: Scalar = Scalar::const_from_int(32);
const NEAREST_PASSABLE_TILE_SEARCH_RADIUS: i32 = 8;
const TANK_SPEED: Scalar = Scalar::const_from_int(120);
const TANK_FOOTPRINT: Scalar = Scalar::const_from_int(14);
const TANK_MAX_HEALTH: f32 = 100.0;

pub struct Sim {
    pub map: Map,
    pub ecs: hecs::World,
    current_tick: Tick,
    pending: PendingCommands,
    log: CommandLog,
}

impl Sim {
    pub fn new(map: Map) -> Self {
        Self {
            map,
            ecs: hecs::World::new(),
            current_tick: 0,
            pending: PendingCommands::default(),
            log: CommandLog::default(),
        }
    }

    /// A stock map with placeholder tanks for smoke tests and the current
    /// single-player entry point.
    pub fn placeholder() -> Self {
        let width = DEFAULT_MAP_WIDTH;
        let height = DEFAULT_MAP_HEIGHT;
        let tiles = (0..(width as usize * height as usize))
            .map(|i| {
                let h = (i as u32).wrapping_mul(2_654_435_761);
                if h.is_multiple_of(12) {
                    Tile::Rock
                } else {
                    Tile::Grass {
                        variant: (i % 3) as u8,
                    }
                }
            })
            .collect();
        let map = Map::new(width, height, tiles).unwrap();
        let mut sim = Self::new(map);
        sim.spawn_placeholder_tanks();
        sim
    }

    #[inline]
    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    #[inline]
    pub fn log(&self) -> &CommandLog {
        &self.log
    }

    pub fn schedule_player_command(&mut self, command: PlayerCommand) {
        let apply_at = self.current_tick + TICK_DELAY_BEFORE_COMMAND_TAKES_EFFECT;
        self.pending
            .schedule_player_command_for_tick(apply_at, command);
    }

    pub fn tick(&mut self, dt: Duration) {
        let ready = self
            .pending
            .drain_player_commands_until_tick(self.current_tick);
        for (tick, cmd) in ready {
            self.apply(&cmd);
            self.log.record(tick, cmd);
        }
        run_movement(&mut self.ecs, &self.map);
        resolve_collisions(&mut self.ecs, &self.map);
        tick_markers(&mut self.ecs, dt);
        self.current_tick += 1;
    }

    fn apply(&mut self, command: &PlayerCommand) {
        match command {
            PlayerCommand::Move { units, target } => self.apply_move(units, *target),
        }
    }

    fn apply_move(&mut self, units: &[hecs::Entity], target: FixedVec2) {
        let existing_move_markers: Vec<hecs::Entity> = self
            .ecs
            .query::<&UnitMovementMarker>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for e in existing_move_markers {
            let _ = self.ecs.despawn(e);
        }

        let nearest_goal_tile = Map::get_world_tile_at(target);
        let Some(goal) = self.map.find_nearest_passable_tile_in_radius(
            nearest_goal_tile,
            NEAREST_PASSABLE_TILE_SEARCH_RADIUS,
        ) else {
            return;
        };

        for &unit in units {
            let start_pos = match self.ecs.get::<&Position>(unit) {
                Ok(p) => p.0,
                Err(_) => continue,
            };
            let start_tile = Map::get_world_tile_at(start_pos);
            let PathFindingResult::Path(tile_path) =
                PathFinder::find_path::<AStarPathFindingAlgorithm>(&self.map, start_tile, goal)
            else {
                continue;
            };
            let Waypoints(waypoints) = Waypoints::compute_from_path(start_pos, target, &tile_path);
            let final_target = *waypoints.last().unwrap_or(&target);
            let _ = self
                .ecs
                .insert_one(unit, UnitMoveInstructions::path(waypoints));
            self.ecs
                .spawn((UnitMovementMarker::new(unit, final_target),));
        }
    }

    fn tank_footprint() -> Footprint {
        Footprint::with_offset(TANK_FOOTPRINT, FixedVec2::ZERO)
    }

    pub fn spawn_placeholder_tanks(&mut self) {
        let center = self.map.world_bounds().center_sim();
        let spacing = Scalar::const_from_int(96);
        for i in -1..=1_i32 {
            let pos = Position(FixedVec2::new(
                center.x + Scalar::from_num(i) * spacing,
                center.y,
            ));
            self.clear_footprint_area(Self::tank_footprint().disc_at(pos));
            self.spawn_tank(pos);
        }
    }

    /// Turn any impassable tiles overlapping `disc` into grass. Used to make
    /// sure the placeholder spawn location is legal.
    fn clear_footprint_area(&mut self, disc: Disc) {
        for c in disc.axis_probes() {
            let (tx, ty) = Map::get_world_tile_at(c.0);
            if tx < 0 || ty < 0 || tx >= self.map.width as i32 || ty >= self.map.height as i32 {
                continue;
            }
            let idx = ty as usize * self.map.width as usize + tx as usize;
            if let Some(t) = self.map.tiles.get_mut(idx)
                && !t.is_passable()
            {
                *t = Tile::Grass { variant: 0 };
            }
        }
    }

    pub fn spawn_tank(&mut self, pos: Position) -> hecs::Entity {
        self.ecs.spawn((
            pos,
            Facing::default(),
            UnitKind::Tank,
            Speed(TANK_SPEED),
            Self::tank_footprint(),
            Name("TANK"),
            Health::new(TANK_MAX_HEALTH),
        ))
    }

    pub fn temp_create_map() -> Map {
        let mut tiles: Vec<Tile> = Vec::new();
        for y in 0u32..64 {
            for x in 0u32..64 {
                if x.is_multiple_of(5) && y.is_multiple_of(3) {
                    tiles.push(Tile::Rock);
                } else if x.is_multiple_of(2) && y.is_multiple_of(9) {
                    tiles.push(Tile::Grass { variant: 1 });
                } else {
                    tiles.push(Tile::Grass { variant: 0 });
                }
            }
        }
        Map::new(64, 64, tiles).unwrap()
    }
}
