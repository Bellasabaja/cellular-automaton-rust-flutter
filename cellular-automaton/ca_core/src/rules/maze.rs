// rules/maze.rs – Maze Regel
//
// Maze erzeugt labyrinthähnliche Strukturen: B3/S12345
//
// Geburt  bei: 3 Nachbarn
// Überleben bei: 1, 2, 3, 4 oder 5 Nachbarn
//
// Warum entstehen Labyrinthe?
//   Zellen überleben sehr lange (bis zu 5 Nachbarn),
//   aber neue Zellen entstehen nur bei genau 3.
//   Das erzeugt lange, verzweigte Korridore.

use crate::grid::{CellState, Grid};
use super::Rule;

pub struct Maze;

impl Rule for Maze {
    fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
        let current   = grid.get(x, y);
        let neighbors = grid.count_alive_neighbors(x, y);

        match current {
            CellState::Alive => {
                // Überleben bei 1–5 Nachbarn
                // `matches!` ist ein praktisches Makro für Bereichsprüfungen
                if matches!(neighbors, 1..=5) {
                    CellState::Alive
                } else {
                    CellState::Dead
                }
            }
            CellState::Dead => {
                // Geburt nur bei genau 3
                if neighbors == 3 {
                    CellState::Alive
                } else {
                    CellState::Dead
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Maze"
    }

    fn description(&self) -> &str {
        "Labyrinth-Regel: B3/S12345. \
         Zellen überleben sehr lange und erzeugen \
         verzweigte Korridore."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, NeighborhoodType};

    #[test]
    fn test_survival_with_one_neighbor() {
        // Maze: lebende Zelle mit 1 Nachbar überlebt (in GoL würde sie sterben)
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Maze;

        grid.set(2, 2, CellState::Alive);
        grid.set(2, 1, CellState::Alive); // 1 Nachbar

        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }

    #[test]
    fn test_death_with_zero_neighbors() {
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Maze;

        grid.set(2, 2, CellState::Alive); // 0 Nachbarn → stirbt
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }

    #[test]
    fn test_death_with_six_neighbors() {
        // Bei 6+ Nachbarn stirbt die Zelle (zu voll)
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Maze;

        grid.set(2, 2, CellState::Alive);
        grid.set(1, 1, CellState::Alive);
        grid.set(2, 1, CellState::Alive);
        grid.set(3, 1, CellState::Alive);
        grid.set(1, 2, CellState::Alive);
        grid.set(3, 2, CellState::Alive);
        grid.set(1, 3, CellState::Alive);

        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }
}