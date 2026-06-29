// rules/mod.rs – Das Rule-Interface
//
// Hier definieren wir den `Rule` Trait – das Herzstück der Erweiterbarkeit.
// Jede konkrete Regel (GameOfLife, HighLife, Maze...) implementiert diesen Trait.
//
// Lernkonzepte:
//   - Traits (Interfaces)
//   - Trait Objects (dyn Rule)
//   - Box<T> (Heap-Allokation)
//   - &self vs &mut self
//   - Dokumentations-Kommentare

// rules/mod.rs – Das Rule-Interface
pub mod game_of_life;
pub mod high_life;
pub mod maze;
pub mod seeds;

use crate::grid::{CellState, Grid};

pub trait Rule: Send + Sync {
    fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuleSet {
    GameOfLife,
    HighLife,
    Maze,
    Seeds,
}

impl RuleSet {
    pub fn to_rule(&self) -> Box<dyn Rule> {
        match self {
            RuleSet::GameOfLife => Box::new(game_of_life::GameOfLife),
            RuleSet::HighLife   => Box::new(high_life::HighLife),
            RuleSet::Maze       => Box::new(maze::Maze),
            RuleSet::Seeds      => Box::new(seeds::Seeds),
        }
    }

    pub fn all() -> Vec<RuleSet> {
        vec![
            RuleSet::GameOfLife,
            RuleSet::HighLife,
            RuleSet::Maze,
            RuleSet::Seeds,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            RuleSet::GameOfLife => "Game of Life",
            RuleSet::HighLife   => "HighLife",
            RuleSet::Maze       => "Maze",
            RuleSet::Seeds      => "Seeds",
        }
    }
}