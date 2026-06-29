// rules/seeds.rs – Seeds Regel
//
// Seeds ist eine der explosivsten Regeln: B2/S
//
// Geburt  bei: genau 2 Nachbarn
// Überleben: NIE (alle lebenden Zellen sterben immer)
//
// Klingt seltsam – aber das Ergebnis ist faszinierend:
// Jede Generation stirbt komplett ab, aber erzeugt
// dabei eine Explosion neuer Zellen. Sehr chaotisch.

use crate::grid::{CellState, Grid};
use super::Rule;

pub struct Seeds;

impl Rule for Seeds {
    fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
        let current   = grid.get(x, y);
        let neighbors = grid.count_alive_neighbors(x, y);

        match current {
            // Lebende Zellen sterben IMMER (keine Überlebensregel)
            CellState::Alive => CellState::Dead,

            // Tote Zellen werden lebendig bei genau 2 Nachbarn
            CellState::Dead => {
                if neighbors == 2 {
                    CellState::Alive
                } else {
                    CellState::Dead
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Seeds"
    }

    fn description(&self) -> &str {
        "Explosive Regel: B2/S. \
         Alle Zellen sterben jede Generation, \
         erzeugen aber chaotisches Wachstum."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, NeighborhoodType};

    #[test]
    fn test_alive_always_dies() {
        // Seeds: lebende Zellen sterben IMMER
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Seeds;

        // Egal wie viele Nachbarn – lebende Zelle stirbt immer
        grid.set(2, 2, CellState::Alive);
        grid.set(2, 1, CellState::Alive);
        grid.set(2, 3, CellState::Alive);

        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }

    #[test]
    fn test_birth_at_exactly_two() {
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Seeds;

        // (2,2) ist tot, hat genau 2 Nachbarn → wird lebendig
        grid.set(1, 1, CellState::Alive);
        grid.set(3, 3, CellState::Alive);

        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }

    #[test]
    fn test_no_birth_at_one_neighbor() {
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = Seeds;

        grid.set(1, 1, CellState::Alive); // nur 1 Nachbar
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Dead);
    }
}