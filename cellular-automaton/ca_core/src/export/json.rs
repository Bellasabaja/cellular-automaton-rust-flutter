// export/json.rs – JSON Export und Import
//
// Speichert den kompletten Simulationszustand als JSON-Datei
// und kann ihn wieder laden.
//
// Lernkonzepte:
//   - serde_json verwenden
//   - Eigene Serialisierungs-Strukturen
//   - Datei schreiben und lesen
//   - Result<T, E> konsequent verwenden

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::grid::{CellState, Grid, NeighborhoodType};
use crate::rules::RuleSet;
use crate::simulator::Simulator;

// =============================================================================
// Snapshot – Der exportierte Zustand
// =============================================================================

/// Ein vollständiger Snapshot der Simulation.
///
/// Diese Struktur wird nach JSON serialisiert und wieder geladen.
/// Sie enthält alles was nötig ist um die Simulation fortzusetzen.
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    /// Metadaten
    pub meta: SnapshotMeta,
    /// Gitter-Zustand
    pub grid: GridSnapshot,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Version des Formats (für spätere Kompatibilität)
    pub version: u32,
    /// Aktuelle Generationsnummer
    pub generation: u64,
    /// Name der aktiven Regel
    pub rule: String,
    /// Anzahl lebendiger Zellen
    pub alive_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSnapshot {
    pub width: usize,
    pub height: usize,
    pub wrap_around: bool,
    /// Nachbarschaftstyp als String
    pub neighborhood: String,
    /// Zellen als flaches Array von 0 (tot) und 1 (lebendig)
    /// Kompakter als "Dead"/"Alive" Strings
    pub cells: Vec<u8>,
}

// =============================================================================
// Fehlertyp
// =============================================================================

#[derive(Debug)]
pub enum JsonExportError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    InvalidData(String),
}

impl std::fmt::Display for JsonExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JsonExportError::IoError(e)      => write!(f, "IO Fehler: {}", e),
            JsonExportError::SerdeError(e)   => write!(f, "JSON Fehler: {}", e),
            JsonExportError::InvalidData(s)  => write!(f, "Ungültige Daten: {}", s),
        }
    }
}

impl From<std::io::Error> for JsonExportError {
    fn from(e: std::io::Error) -> Self { JsonExportError::IoError(e) }
}

impl From<serde_json::Error> for JsonExportError {
    fn from(e: serde_json::Error) -> Self { JsonExportError::SerdeError(e) }
}

// =============================================================================
// Export
// =============================================================================

/// Exportiert den aktuellen Simulator-Zustand als JSON-Datei.
pub fn export_json(
    simulator: &Simulator,
    path: &Path,
    rule_name: &str,
) -> Result<(), JsonExportError> {
    let snapshot = build_snapshot(simulator, rule_name);
    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(path, json)?;
    Ok(())
}

/// Exportiert als JSON-String (nützlich für Tests und Web).
pub fn export_json_string(
    simulator: &Simulator,
    rule_name: &str,
) -> Result<String, JsonExportError> {
    let snapshot = build_snapshot(simulator, rule_name);
    Ok(serde_json::to_string_pretty(&snapshot)?)
}

/// Baut einen Snapshot aus dem aktuellen Simulator-Zustand.
fn build_snapshot(simulator: &Simulator, rule_name: &str) -> SimulationSnapshot {
    let grid = simulator.grid();

    // CellState → u8: Alive = 1, Dead = 0
    // collect() wandelt einen Iterator in einen Vec um
    let cells: Vec<u8> = grid.cells()
        .iter()
        .map(|c| if c.is_alive() { 1 } else { 0 })
        .collect();

    let neighborhood = match grid.neighborhood {
        NeighborhoodType::Moore      => "moore",
        NeighborhoodType::VonNeumann => "von_neumann",
    };

    SimulationSnapshot {
        meta: SnapshotMeta {
            version:     1,
            generation:  simulator.generation(),
            rule:        rule_name.to_string(),
            alive_count: grid.count_alive(),
        },
        grid: GridSnapshot {
            width:        grid.width,
            height:       grid.height,
            wrap_around:  grid.wrap_around,
            neighborhood: neighborhood.to_string(),
            cells,
        },
    }
}

// =============================================================================
// Import
// =============================================================================

/// Lädt einen Snapshot aus einer JSON-Datei.
pub fn import_json(path: &Path) -> Result<SimulationSnapshot, JsonExportError> {
    let content = fs::read_to_string(path)?;
    import_json_string(&content)
}

/// Lädt einen Snapshot aus einem JSON-String.
pub fn import_json_string(json: &str) -> Result<SimulationSnapshot, JsonExportError> {
    let snapshot: SimulationSnapshot = serde_json::from_str(json)?;

    // Validierung
    let expected = snapshot.grid.width * snapshot.grid.height;
    if snapshot.grid.cells.len() != expected {
        return Err(JsonExportError::InvalidData(format!(
            "Zell-Array hat {} Einträge, erwartet {}",
            snapshot.grid.cells.len(), expected
        )));
    }

    Ok(snapshot)
}

/// Baut ein Grid aus einem Snapshot.
pub fn snapshot_to_grid(snapshot: &SimulationSnapshot)
    -> Result<Grid, JsonExportError>
{
    use crate::config::parse_neighborhood;

    let neighborhood = parse_neighborhood(&snapshot.grid.neighborhood)
        .map_err(|e| JsonExportError::InvalidData(e.to_string()))?;

    let mut grid = Grid::new(
        snapshot.grid.width,
        snapshot.grid.height,
        snapshot.grid.wrap_around,
        neighborhood,
    );

    // u8 → CellState zurückkonvertieren
    for (i, &cell_val) in snapshot.grid.cells.iter().enumerate() {
        let x = (i % snapshot.grid.width) as isize;
        let y = (i / snapshot.grid.width) as isize;
        let state = if cell_val == 1 {
            CellState::Alive
        } else {
            CellState::Dead
        };
        grid.set(x, y, state);
    }

    Ok(grid)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::NeighborhoodType;
    use crate::rules::RuleSet;
    use crate::simulator::{Simulator, SimulatorConfig};

    fn make_simulator() -> Simulator {
        Simulator::new(SimulatorConfig {
            width: 10,
            height: 10,
            wrap_around: true,
            neighborhood: NeighborhoodType::Moore,
            rule_set: RuleSet::GameOfLife,
            history_size: 0,
            random_start: None,
        })
    }

    #[test]
    fn test_export_and_import_roundtrip() {
        let mut sim = make_simulator();

        // Paar Zellen setzen
        sim.grid_mut().set(1, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.grid_mut().set(3, 3, CellState::Alive);
        sim.step(5);

        // Export
        let json = export_json_string(&sim, "game_of_life").unwrap();

        // Import
        let snapshot = import_json_string(&json).unwrap();

        // Metadaten prüfen
        assert_eq!(snapshot.meta.generation, 5);
        assert_eq!(snapshot.meta.rule, "game_of_life");
        assert_eq!(snapshot.grid.width, 10);
        assert_eq!(snapshot.grid.height, 10);
        assert_eq!(snapshot.grid.cells.len(), 100);
    }

    #[test]
    fn test_snapshot_to_grid_preserves_state() {
        let mut sim = make_simulator();
        sim.grid_mut().set(5, 5, CellState::Alive);
        sim.grid_mut().set(6, 6, CellState::Alive);

        let json    = export_json_string(&sim, "game_of_life").unwrap();
        let snapshot = import_json_string(&json).unwrap();
        let grid    = snapshot_to_grid(&snapshot).unwrap();

        // Zellen müssen nach Import noch stimmen
        assert_eq!(grid.get(5, 5), CellState::Alive);
        assert_eq!(grid.get(6, 6), CellState::Alive);
        assert_eq!(grid.get(0, 0), CellState::Dead);
    }

    #[test]
    fn test_import_invalid_cell_count() {
        // Absichtlich kaputtes JSON
        let bad_json = r#"{
            "meta": {
                "version": 1,
                "generation": 0,
                "rule": "game_of_life",
                "alive_count": 0
            },
            "grid": {
                "width": 10,
                "height": 10,
                "wrap_around": true,
                "neighborhood": "moore",
                "cells": [0, 1, 0]
            }
        }"#;

        // Sollte einen InvalidData Fehler geben
        let result = import_json_string(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_alive_count_in_snapshot() {
        let mut sim = make_simulator();
        sim.grid_mut().set(1, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);

        let json     = export_json_string(&sim, "game_of_life").unwrap();
        let snapshot = import_json_string(&json).unwrap();

        assert_eq!(snapshot.meta.alive_count, 2);
    }
}