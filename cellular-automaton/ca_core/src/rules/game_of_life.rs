// rules/game_of_life.rs – Conways Game of Life
//
// Die klassische Regel, 1970 von John Conway erfunden.
//
// Regeln:
//   1. Eine lebende Zelle mit 2 oder 3 lebenden Nachbarn überlebt.
//   2. Eine tote Zelle mit genau 3 lebenden Nachbarn wird lebendig.
//   3. Alle anderen Zellen sterben oder bleiben tot.
//
// Kurzschreibweise: B3/S23
//   B = Born  (wird lebendig bei N Nachbarn)
//   S = Survive (überlebt bei N Nachbarn)

use crate::grid::{CellState, Grid};
use super::Rule;

// =============================================================================
// GameOfLife Struct
// =============================================================================

/// Implementierung der klassischen Game-of-Life-Regel.
///
/// Das Struct hat keine Felder – die Regel ist zustandslos.
/// Alle nötigen Informationen kommen vom Grid.
pub struct GameOfLife;

impl Rule for GameOfLife {
    fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
        let current   = grid.get(x, y);
        let neighbors = grid.count_alive_neighbors(x, y);

        match (current, neighbors) {
            // Regel 1: Lebende Zelle mit 2 oder 3 Nachbarn überlebt
            (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive,

            // Regel 2: Tote Zelle mit genau 3 Nachbarn wird lebendig
            (CellState::Dead, 3) => CellState::Alive,

            // Regel 3: Alles andere stirbt oder bleibt tot
            _ => CellState::Dead,
        }
    }

    fn name(&self) -> &str {
        "Game of Life"
    }

    fn description(&self) -> &str {
        "Klassische Conway-Regel: B3/S23. \
         Lebende Zellen überleben mit 2-3 Nachbarn, \
         tote Zellen werden mit genau 3 Nachbarn lebendig."
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, NeighborhoodType};

    /// Hilfsfunktion: erstellt ein 5x5 Grid mit einer lebenden Zelle in der Mitte
    fn single_cell_grid() -> Grid {
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        grid.set(2, 2, CellState::Alive);
        grid
    }

    #[test]
    fn test_underpopulation() {
        // Eine lebende Zelle mit 0 oder 1 Nachbar stirbt
        let grid = single_cell_grid();
        let rule = GameOfLife;

        // Zelle (2,2) hat 0 Nachbarn → stirbt
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }

    #[test]
    fn test_survival() {
        // Eine lebende Zelle mit 2 oder 3 Nachbarn überlebt
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = GameOfLife;

        // Zelle (2,2) mit 2 Nachbarn → überlebt
        grid.set(2, 2, CellState::Alive);
        grid.set(1, 1, CellState::Alive);
        grid.set(3, 3, CellState::Alive);
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }

    #[test]
    fn test_overpopulation() {
        // Eine lebende Zelle mit 4+ Nachbarn stirbt
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = GameOfLife;

        grid.set(2, 2, CellState::Alive);
        grid.set(1, 1, CellState::Alive);
        grid.set(2, 1, CellState::Alive);
        grid.set(3, 1, CellState::Alive);
        grid.set(1, 2, CellState::Alive);
        // (2,2) hat jetzt 4 Nachbarn → stirbt
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }

    #[test]
    fn test_reproduction() {
        // Eine tote Zelle mit genau 3 Nachbarn wird lebendig
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = GameOfLife;

        // (2,2) ist tot, hat 3 lebende Nachbarn → wird lebendig
        grid.set(1, 1, CellState::Alive);
        grid.set(2, 1, CellState::Alive);
        grid.set(3, 1, CellState::Alive);
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }

    #[test]
    fn test_blinker_pattern() {
        // Der Blinker ist das einfachste Oszillator-Muster in GoL:
        // Generation 0:          Generation 1:
        //   . . . . .              . . . . .
        //   . . X . .              . . X . .
        //   . . X . .    →         . . X . .
        //   . . X . .              . . X . .
        //   . . . . .              . . . . .
        // Drei vertikale Zellen → drei horizontale Zellen

        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = GameOfLife;

        // Vertikaler Blinker
        grid.set(2, 1, CellState::Alive);
        grid.set(2, 2, CellState::Alive);
        grid.set(2, 3, CellState::Alive);

        // Nach einem Tick sollte (1,2) und (3,2) lebendig sein
        assert_eq!(rule.next_state(&grid, 1, 2), CellState::Alive);
        assert_eq!(rule.next_state(&grid, 3, 2), CellState::Alive);
        // Und (2,1) und (2,3) sollten sterben
        assert_eq!(rule.next_state(&grid, 2, 1), CellState::Dead);
        assert_eq!(rule.next_state(&grid, 2, 3), CellState::Dead);
    }
}