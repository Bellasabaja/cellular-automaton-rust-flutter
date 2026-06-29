// export/csv.rs – CSV Export
//
// Exportiert den Populationsverlauf als CSV-Datei.
// Nützlich für Analysen in Excel, Python, R etc.
//
// Format:
//   generation,alive_count,delta
//   0,150,0
//   1,142,-8
//   2,138,-4
//   ...
//
// Lernkonzepte:
//   - String-Formatierung mit format!()
//   - Vec<String> zu einem String zusammenführen (join)
//   - Iteratoren mit enumerate()

use std::fs;
use std::path::Path;

use crate::simulator::{GenerationStats, Simulator};

// =============================================================================
// Fehlertyp
// =============================================================================

#[derive(Debug)]
pub enum CsvExportError {
    IoError(std::io::Error),
}

impl std::fmt::Display for CsvExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CsvExportError::IoError(e) => write!(f, "IO Fehler: {}", e),
        }
    }
}

impl From<std::io::Error> for CsvExportError {
    fn from(e: std::io::Error) -> Self {
        CsvExportError::IoError(e)
    }
}

// =============================================================================
// Export
// =============================================================================

/// Exportiert den Populationsverlauf aus der History als CSV-Datei.
pub fn export_csv(simulator: &Simulator, path: &Path) -> Result<(), CsvExportError> {
    let content = build_csv(simulator.stats_history());
    fs::write(path, content)?;
    Ok(())
}

/// Gibt den CSV-Inhalt als String zurück (nützlich für Tests).
pub fn export_csv_string(simulator: &Simulator) -> String {
    build_csv(simulator.stats_history())
}

/// Baut den CSV-String aus der Stats-History.
fn build_csv(history: &[GenerationStats]) -> String {
    // Header-Zeile
    let mut lines = vec![
        "generation,alive_count,delta".to_string()
    ];

    // Eine Zeile pro Generation
    // `iter()` gibt Referenzen, kein Ownership-Problem
    for stats in history.iter() {
        lines.push(format!(
            "{},{},{}",
            stats.generation,
            stats.alive_count,
            stats.delta
        ));
    }

    // Alle Zeilen mit Newline verbinden
    // `join()` fügt einen Trenner zwischen alle Elemente ein
    lines.join("\n")
}

/// Exportiert den aktuellen Gitterzustand als CSV-Matrix.
///
/// Jede Zeile entspricht einer Gitterzeile.
/// 0 = tot, 1 = lebendig.
/// Nützlich für externe Analyse-Tools.
pub fn export_grid_csv(simulator: &Simulator, path: &Path) -> Result<(), CsvExportError> {
    let content = build_grid_csv(simulator);
    fs::write(path, content)?;
    Ok(())
}

/// Gibt den Gitter-CSV als String zurück.
pub fn export_grid_csv_string(simulator: &Simulator) -> String {
    build_grid_csv(simulator)
}

/// Baut den CSV-String für das Gitter.
fn build_grid_csv(simulator: &Simulator) -> String {
    let grid = simulator.grid();
    let mut lines = Vec::new();

    for y in 0..grid.height {
        // Jede Zeile: Werte mit Komma getrennt
        let row: Vec<String> = (0..grid.width)
            .map(|x| {
                if grid.get(x as isize, y as isize).is_alive() {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            })
            .collect();

        lines.push(row.join(","));
    }

    lines.join("\n")
}

// =============================================================================
// Import (Populationsverlauf)
// =============================================================================

/// Parsed einen CSV-String zurück in GenerationStats.
/// Nützlich für Tests und spätere Analyse-Features.
pub fn parse_csv_string(csv: &str) -> Result<Vec<GenerationStats>, String> {
    let mut result = Vec::new();

    for (line_num, line) in csv.lines().enumerate() {
        // Header überspringen
        if line_num == 0 {
            continue;
        }

        // Leere Zeilen überspringen
        if line.trim().is_empty() {
            continue;
        }

        // Spalten aufteilen
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Zeile {}: erwartet 3 Spalten, gefunden {}", line_num, parts.len()));
        }

        // Strings zu Zahlen parsen
        // `parse::<u64>()` = expliziter Typ-Parameter für parse()
        let generation = parts[0].trim().parse::<u64>()
            .map_err(|e| format!("Zeile {}: generation ungültig: {}", line_num, e))?;
        let alive_count = parts[1].trim().parse::<usize>()
            .map_err(|e| format!("Zeile {}: alive_count ungültig: {}", line_num, e))?;
        let delta = parts[2].trim().parse::<i64>()
            .map_err(|e| format!("Zeile {}: delta ungültig: {}", line_num, e))?;

        result.push(GenerationStats { generation, alive_count, delta });
    }

    Ok(result)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellState, NeighborhoodType};
    use crate::rules::RuleSet;
    use crate::simulator::{Simulator, SimulatorConfig};

    fn make_simulator() -> Simulator {
        Simulator::new(SimulatorConfig {
            width: 5,
            height: 5,
            wrap_around: false,
            neighborhood: NeighborhoodType::Moore,
            rule_set: RuleSet::GameOfLife,
            history_size: 100,
            random_start: None,
        })
    }

    #[test]
    fn test_csv_has_header() {
        let sim = make_simulator();
        let csv = export_csv_string(&sim);
        assert!(csv.starts_with("generation,alive_count,delta"));
    }

    #[test]
    fn test_csv_empty_history() {
        let sim = make_simulator();
        let csv = export_csv_string(&sim);
        // Nur Header, keine Datenzeilen
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_csv_after_steps() {
        let mut sim = make_simulator();

        // Blinker setzen (3 Zellen)
        sim.grid_mut().set(2, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.grid_mut().set(2, 3, CellState::Alive);

        sim.step(3);

        let csv = export_csv_string(&sim);
        let lines: Vec<&str> = csv.lines().collect();

        // Header + 3 Datenzeilen
        assert_eq!(lines.len(), 4);
        // Erste Datenzeile beginnt mit "1,"
        assert!(lines[1].starts_with("1,"));
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut sim = make_simulator();

        sim.grid_mut().set(2, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.grid_mut().set(2, 3, CellState::Alive);
        sim.step(3);

        let csv    = export_csv_string(&sim);
        let parsed = parse_csv_string(&csv).unwrap();

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].generation, 1);
        // Blinker hat immer 3 lebende Zellen
        assert_eq!(parsed[0].alive_count, 3);
    }

    #[test]
    fn test_grid_csv_dimensions() {
        let sim = make_simulator();
        let csv = export_grid_csv_string(&sim);
        let lines: Vec<&str> = csv.lines().collect();

        // 5 Zeilen (height = 5)
        assert_eq!(lines.len(), 5);
        // Jede Zeile hat 5 Werte (width = 5), getrennt durch 4 Kommas
        assert_eq!(lines[0].split(',').count(), 5);
    }

    #[test]
    fn test_grid_csv_values() {
        let mut sim = make_simulator();
        sim.grid_mut().set(0, 0, CellState::Alive);

        let csv   = export_grid_csv_string(&sim);
        let lines: Vec<&str> = csv.lines().collect();

        // Erste Zeile, erste Zelle = 1
        assert!(lines[0].starts_with("1,"));
        // Zweite Zeile beginnt mit 0
        assert!(lines[1].starts_with("0,"));
    }
}