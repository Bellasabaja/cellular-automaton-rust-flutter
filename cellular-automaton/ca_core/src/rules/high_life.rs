// rules/high_life.rs – HighLife Regel
//
// HighLife ist eine Variante von Game of Life mit einer zusätzlichen
// Geburtsregel: B36/S23
//
// Unterschied zu GoL:
//   - Geburt auch bei 6 Nachbarn (zusätzlich zu 3)
//   - Überleben gleich wie GoL (2 oder 3 Nachbarn)
//
// Interessant weil: HighLife hat einen "Replikator" – ein Muster
// das sich selbst kopiert. In GoL gibt es sowas nicht.

use crate::grid::{CellState, Grid};
use super::Rule;

pub struct HighLife;

impl Rule for HighLife {
    fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
        let current   = grid.get(x, y);
        let neighbors = grid.count_alive_neighbors(x, y);

        match (current, neighbors) {
            // Überleben: 2 oder 3 Nachbarn (identisch zu GoL)
            (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive,

            // Geburt: 3 ODER 6 Nachbarn (das ist der Unterschied zu GoL)
            (CellState::Dead, 3) | (CellState::Dead, 6) => CellState::Alive,

            _ => CellState::Dead,
        }
    }

    fn name(&self) -> &str {
        "HighLife"
    }

    fn description(&self) -> &str {
        "Variante von GoL: B36/S23. \
         Zusätzliche Geburtsregel bei 6 Nachbarn erzeugt \
         selbst-replizierende Muster."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, NeighborhoodType};

    #[test]
    fn test_survival_same_as_gol() {
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = HighLife;

        // 2 Nachbarn → überlebt (wie GoL)
        grid.set(2, 2, CellState::Alive);
        grid.set(1, 1, CellState::Alive);
        grid.set(3, 3, CellState::Alive);
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }

    #[test]
    fn test_birth_at_6_neighbors() {
        // Der entscheidende Unterschied zu GoL:
        // Eine tote Zelle mit 6 Nachbarn wird lebendig
        let mut grid = Grid::new(5, 5, false, NeighborhoodType::Moore);
        let rule = HighLife;

        // (2,2) ist tot – 6 Nachbarn rundherum setzen
        grid.set(1, 1, CellState::Alive);
        grid.set(2, 1, CellState::Alive);
        grid.set(3, 1, CellState::Alive);
        grid.set(1, 2, CellState::Alive);
        grid.set(3, 2, CellState::Alive);
        grid.set(1, 3, CellState::Alive);

        // In GoL würde (2,2) tot bleiben (6 Nachbarn → kein Spawn)
        // In HighLife wird (2,2) lebendig
        assert_eq!(rule.next_state(&grid, 2, 2), CellState::Alive);
    }
}