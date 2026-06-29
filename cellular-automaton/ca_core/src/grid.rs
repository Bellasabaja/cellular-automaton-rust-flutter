// grid.rs – Das Gitter des Zellulären Automaten
//
// Das Grid ist die zentrale Datenstruktur. Es speichert den Zustand
// aller Zellen und stellt Methoden bereit, um Zellen zu lesen,
// zu schreiben und Nachbarn zu berechnen.
//
// Lernkonzepte in dieser Datei:
//   - Structs (Datenstrukturen)
//   - Enums (Aufzählungstypen)
//   - impl-Blöcke (Methoden an Structs)
//   - Ownership & Borrowing
//   - Iteratoren
//   - serde (Serialisierung für JSON-Export)

use serde::{Deserialize, Serialize};

// =============================================================================
// CellState – Was kann eine Zelle sein?
// =============================================================================

/// Der Zustand einer einzelnen Zelle.
///
/// Wir verwenden u8 (unsigned 8-bit integer) als Basis, damit wir
/// später auch Zustände mit Werten (z. B. Alter einer Zelle) darstellen können.
///
/// `#[derive(...)]` lässt Rust automatisch nützliche Traits implementieren:
///   - Clone / Copy  → Wert kann kopiert werden (wichtig für das Grid)
///   - PartialEq     → Vergleich mit == möglich
///   - Debug         → Ausgabe mit {:?} möglich
///   - Serialize /
///     Deserialize   → JSON Export/Import mit serde
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum CellState {
    /// Die Zelle ist tot (inaktiv)
    Dead,
    /// Die Zelle ist lebendig (aktiv)
    Alive,
}

impl CellState {
    /// Gibt true zurück wenn die Zelle lebendig ist.
    /// Praktische Hilfsmethode statt überall `== CellState::Alive` zu schreiben.
    pub fn is_alive(&self) -> bool {
        *self == CellState::Alive
    }

    /// Kehrt den Zustand um: Alive → Dead, Dead → Alive.
    /// Nützlich für Toggle-Interaktionen im Frontend.
    pub fn toggle(&self) -> CellState {
        match self {
            CellState::Alive => CellState::Dead,
            CellState::Dead  => CellState::Alive,
        }
    }
}

// =============================================================================
// NeighborhoodType – Welche Nachbarn zählen?
// =============================================================================

/// Definiert welche Nachbarzellen berücksichtigt werden.
///
/// Das ist einer der Parameter, der das Verhalten der Simulation
/// grundlegend verändert – ohne eine einzige Regel anzupassen.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum NeighborhoodType {
    /// Moore-Nachbarschaft: alle 8 Nachbarn (Standard für Game of Life)
    /// X X X
    /// X O X
    /// X X X
    Moore,

    /// Von-Neumann-Nachbarschaft: nur 4 direkte Nachbarn (oben, unten, links, rechts)
    ///   X
    /// X O X
    ///   X
    VonNeumann,
}

// =============================================================================
// Grid – Das Gitter selbst
// =============================================================================

/// Das zweidimensionale Gitter aller Zellen.
///
/// Intern speichern wir die Zellen als eindimensionalen Vec (Vektor/Array).
/// Der Zugriff auf Zelle (x, y) erfolgt über den Index: `y * width + x`
///
/// Warum 1D statt 2D?
///   - Effizienter im Speicher (zusammenhängend)
///   - Einfacher zu serialisieren (JSON)
///   - Rust's Vec<Vec<T>> wäre weniger performant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    /// Breite des Gitters (Anzahl Spalten)
    pub width: usize,

    /// Höhe des Gitters (Anzahl Zeilen)
    pub height: usize,

    /// Alle Zellen als flaches Array: Länge = width * height
    /// Index einer Zelle (x, y) = y * width + x
    cells: Vec<CellState>,

    /// Ob die Ränder des Gitters "wrappen" (torusförmig verbunden sind)
    /// true  → rechter Rand ist mit linkem verbunden, oben mit unten
    /// false → Zellen am Rand haben weniger Nachbarn
    pub wrap_around: bool,

    /// Welche Nachbarschaft verwendet wird
    pub neighborhood: NeighborhoodType,
}

impl Grid {
    // =========================================================================
    // Konstruktoren
    // =========================================================================

    /// Erstellt ein neues, leeres Gitter (alle Zellen = Dead).
    ///
    /// # Parameter
    /// - `width`       : Breite in Zellen
    /// - `height`      : Höhe in Zellen
    /// - `wrap_around` : Ob Ränder verbunden sind
    /// - `neighborhood`: Welche Nachbarschaft verwendet wird
    ///
    /// # Beispiel
    /// ```rust
    /// use ca_core::grid::{Grid, NeighborhoodType};
    /// let grid = Grid::new(100, 100, true, NeighborhoodType::Moore);
    /// ```
    pub fn new(
        width: usize,
        height: usize,
        wrap_around: bool,
        neighborhood: NeighborhoodType,
    ) -> Self {
        // vec![wert; anzahl] → erstellt einen Vektor mit `anzahl` mal `wert`
        let cells = vec![CellState::Dead; width * height];

        Grid {
            width,
            height,
            cells,
            wrap_around,
            neighborhood,
        }
    }

    /// Erstellt ein Gitter und füllt es zufällig.
    ///
    /// `density` ist ein Wert zwischen 0.0 und 1.0:
    ///   0.0 = alle tot, 1.0 = alle lebendig, 0.5 = ~50% lebendig
    ///
    /// Wir verwenden keine externe Zufalls-Crate, sondern einen einfachen
    /// LCG (Linear Congruential Generator) um die Abhängigkeiten gering zu halten.
    pub fn new_random(
        width: usize,
        height: usize,
        wrap_around: bool,
        neighborhood: NeighborhoodType,
        density: f64,
        seed: u64,
    ) -> Self {
        let mut grid = Grid::new(width, height, wrap_around, neighborhood);

        // LCG Zufallsgenerator (einfach, deterministisch, seed-basiert)
        // Gleicher Seed → gleiche Ausgabe (gut für reproduzierbare Tests)
        let mut rng = seed;
        for i in 0..grid.cells.len() {
            // LCG Formel: next = (a * current + c) % m
            rng = rng.wrapping_mul(6_364_136_223_846_793_005)
                     .wrapping_add(1_442_695_040_888_963_407);

            // Normieren auf [0.0, 1.0) und mit density vergleichen
            let value = (rng >> 32) as f64 / (u32::MAX as f64);
            grid.cells[i] = if value < density {
                CellState::Alive
            } else {
                CellState::Dead
            };
        }

        grid
    }

    // =========================================================================
    // Zugriff auf Zellen
    // =========================================================================

    /// Berechnet den flachen Index für Koordinaten (x, y).
    ///
    /// Internes Hilfsmittel – nicht nach außen sichtbar (kein `pub`).
    ///
    /// Gibt `None` zurück wenn die Koordinaten außerhalb des Gitters liegen
    /// und wrap_around deaktiviert ist.
    fn index(&self, x: isize, y: isize) -> Option<usize> {
        if self.wrap_around {
            // Modulo mit wrap: -1 wird zu width-1, width wird zu 0
            // rem_euclid ist wichtig: Rust's % kann negative Werte liefern
            let wx = x.rem_euclid(self.width as isize) as usize;
            let wy = y.rem_euclid(self.height as isize) as usize;
            Some(wy * self.width + wx)
        } else {
            // Ohne wrap: außerhalb des Gitters → None
            if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
                None
            } else {
                Some(y as usize * self.width + x as usize)
            }
        }
    }

    /// Gibt den Zustand der Zelle an Position (x, y) zurück.
    ///
    /// Gibt `CellState::Dead` zurück wenn die Position außerhalb liegt
    /// (und wrap_around = false). Das vereinfacht die Regellogik enorm.
    pub fn get(&self, x: isize, y: isize) -> CellState {
        match self.index(x, y) {
            Some(idx) => self.cells[idx],
            None      => CellState::Dead, // Außerhalb = tot
        }
    }

    /// Setzt den Zustand der Zelle an Position (x, y).
    ///
    /// Macht nichts wenn die Position außerhalb liegt.
    pub fn set(&mut self, x: isize, y: isize, state: CellState) {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx] = state;
        }
    }

    /// Kehrt den Zustand der Zelle an Position (x, y) um.
    /// (Alive → Dead, Dead → Alive)
    pub fn toggle(&mut self, x: isize, y: isize) {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx] = self.cells[idx].toggle();
        }
    }

    // =========================================================================
    // Nachbarn
    // =========================================================================

    /// Gibt alle Nachbar-Koordinaten einer Zelle zurück.
    ///
    /// Je nach `neighborhood` werden unterschiedliche Offsets verwendet.
    /// Die Koordinaten können negativ sein oder größer als width/height –
    /// das handhabt `get()` durch wrap_around oder None.
    fn neighbor_offsets(&self) -> &[(isize, isize)] {
        match self.neighborhood {
            NeighborhoodType::Moore => &[
                (-1, -1), (0, -1), (1, -1),
                (-1,  0),          (1,  0),
                (-1,  1), (0,  1), (1,  1),
            ],
            NeighborhoodType::VonNeumann => &[
                          (0, -1),
                (-1,  0),          (1,  0),
                          (0,  1),
            ],
        }
    }

    /// Zählt die lebendigen Nachbarn einer Zelle an Position (x, y).
    ///
    /// Das ist die meistgenutzte Methode in jeder Regel-Implementierung.
    pub fn count_alive_neighbors(&self, x: isize, y: isize) -> u8 {
        self.neighbor_offsets()
            .iter()
            // Für jeden Offset: absolute Koordinate berechnen
            .map(|(dx, dy)| self.get(x + dx, y + dy))
            // Nur lebendige Nachbarn zählen
            .filter(|state| state.is_alive())
            .count() as u8
    }

    // =========================================================================
    // Gitter-Operationen
    // =========================================================================

    /// Setzt alle Zellen auf Dead (leeres Gitter).
    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = CellState::Dead;
        }
    }

    /// Ändert die Größe des Gitters.
    ///
    /// Bestehende Zellen werden so gut wie möglich übernommen.
    /// Neue Zellen werden auf Dead gesetzt.
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let mut new_cells = vec![CellState::Dead; new_width * new_height];

        // Bestehende Zellen kopieren (nur der überlappende Bereich)
        let copy_width  = self.width.min(new_width);
        let copy_height = self.height.min(new_height);

        for y in 0..copy_height {
            for x in 0..copy_width {
                let old_idx = y * self.width + x;
                let new_idx = y * new_width + x;
                new_cells[new_idx] = self.cells[old_idx];
            }
        }

        self.width  = new_width;
        self.height = new_height;
        self.cells  = new_cells;
    }

    /// Gibt die Gesamtzahl der lebendigen Zellen zurück.
    /// Nützlich für den CSV-Export (Populationsverlauf).
    pub fn count_alive(&self) -> usize {
        self.cells.iter().filter(|c| c.is_alive()).count()
    }

    /// Gibt eine Referenz auf alle Zellen zurück (für den Renderer).
    /// `&[T]` ist ein "slice" – eine Referenz auf einen Teil eines Arrays.
    pub fn cells(&self) -> &[CellState] {
        &self.cells
    }

    /// Füllt einen rechteckigen Bereich mit einem bestimmten Zustand.
    /// Nützlich für Pattern-Placement und Zone-Initialisierung.
    pub fn fill_rect(
        &mut self,
        x: isize, y: isize,
        w: usize, h: usize,
        state: CellState,
    ) {
        for dy in 0..h as isize {
            for dx in 0..w as isize {
                self.set(x + dx, y + dy, state);
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

/// Tests werden mit `cargo test` ausgeführt.
/// `#[cfg(test)]` bedeutet: dieser Block wird nur beim Testen kompiliert.
#[cfg(test)]
mod tests {
    // `super::*` importiert alles aus dem Eltern-Modul (grid.rs)
    use super::*;

    #[test]
    fn test_new_grid_is_empty() {
        let grid = Grid::new(10, 10, true, NeighborhoodType::Moore);
        // Alle Zellen müssen tot sein
        assert_eq!(grid.count_alive(), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut grid = Grid::new(10, 10, true, NeighborhoodType::Moore);
        grid.set(3, 4, CellState::Alive);
        assert_eq!(grid.get(3, 4), CellState::Alive);
        assert_eq!(grid.get(0, 0), CellState::Dead);
    }

    #[test]
    fn test_wrap_around() {
         let mut grid = Grid::new(10, 10, true, NeighborhoodType::Moore);

         // (10, 0) muss auf (0, 0) mappen
         grid.set(0, 0, CellState::Alive);
         assert_eq!(grid.get(10, 0), CellState::Alive);

         // (-1, 0) muss auf (9, 0) mappen
         grid.set(9, 0, CellState::Alive);
         assert_eq!(grid.get(-1, 0), CellState::Alive);
    }

    #[test]
    fn test_no_wrap_around() {
        let mut grid = Grid::new(10, 10, false, NeighborhoodType::Moore);
        grid.set(0, 0, CellState::Alive);
        // Ohne wrap: außerhalb = Dead
        assert_eq!(grid.get(-1, 0), CellState::Dead);
    }

    #[test]
    fn test_count_alive_neighbors() {
        let mut grid = Grid::new(10, 10, true, NeighborhoodType::Moore);
        // Mittlere Zelle (5,5) hat 0 lebendige Nachbarn
        assert_eq!(grid.count_alive_neighbors(5, 5), 0);
        // Einen Nachbarn setzen
        grid.set(4, 4, CellState::Alive);
        assert_eq!(grid.count_alive_neighbors(5, 5), 1);
        // Noch einen
        grid.set(5, 4, CellState::Alive);
        assert_eq!(grid.count_alive_neighbors(5, 5), 2);
    }

    #[test]
    fn test_toggle() {
        let mut grid = Grid::new(10, 10, true, NeighborhoodType::Moore);
        grid.toggle(3, 3);
        assert_eq!(grid.get(3, 3), CellState::Alive);
        grid.toggle(3, 3);
        assert_eq!(grid.get(3, 3), CellState::Dead);
    }

    #[test]
    fn test_resize_keeps_existing_cells() {
        let mut grid = Grid::new(5, 5, true, NeighborhoodType::Moore);
        grid.set(2, 2, CellState::Alive);
        grid.resize(10, 10);
        // Zelle muss nach resize noch da sein
        assert_eq!(grid.get(2, 2), CellState::Alive);
        // Neue Zellen müssen tot sein
        assert_eq!(grid.get(7, 7), CellState::Dead);
    }
}