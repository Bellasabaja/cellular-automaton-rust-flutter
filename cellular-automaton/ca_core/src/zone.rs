// zone.rs – Zonen-Verwaltung
//
// Eine Zone ist ein rechteckiger Bereich des Gitters mit einer eigenen Regel.
// Der ZoneManager verwaltet alle Zonen und entscheidet für jede Zelle
// welche Regel angewendet wird.
//
// Lernkonzepte:
//   - Structs mit Methoden
//   - Vec<T> als dynamische Liste
//   - Iteration über eine Collection
//   - Option<T> als Rückgabewert
//   - Prioritäten (Zonen können sich überlappen)

use crate::rules::{Rule, RuleSet};
use crate::grid::{CellState, Grid};

// =============================================================================
// Zone – Ein Bereich mit eigener Regel
// =============================================================================

/// Eine rechteckige Zone auf dem Gitter mit einer eigenen Regel.
///
/// Zonen können sich überlappen – in diesem Fall gewinnt die Zone
/// mit der höchsten Priorität (niedrigste Zahl = höchste Priorität).
pub struct Zone {
    /// Eindeutiger Name der Zone (für UI und Export)
    pub name: String,

    /// Linke obere Ecke (x, y)
    pub x: isize,
    pub y: isize,

    /// Größe der Zone
    pub width: usize,
    pub height: usize,

    /// Die Regel die in dieser Zone gilt
    pub rule_set: RuleSet,

    /// Priorität bei Überlappung (0 = höchste Priorität)
    pub priority: u32,

    /// Ob die Zone aktiv ist
    pub active: bool,
}

impl Zone {
    /// Erstellt eine neue Zone.
    pub fn new(
        name: impl Into<String>,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        rule_set: RuleSet,
        priority: u32,
    ) -> Self {
        Zone {
            name: name.into(),
            x,
            y,
            width,
            height,
            rule_set,
            priority,
            active: true,
        }
    }

    /// Prüft ob eine Koordinate (px, py) innerhalb dieser Zone liegt.
    pub fn contains(&self, px: isize, py: isize) -> bool {
        px >= self.x
            && py >= self.y
            && px < self.x + self.width as isize
            && py < self.y + self.height as isize
    }
}

// =============================================================================
// ZoneManager – Verwaltet alle Zonen
// =============================================================================

/// Verwaltet eine Liste von Zonen und die Standard-Regel.
///
/// Für jede Zelle wird die Regel so bestimmt:
///   1. Alle aktiven Zonen prüfen ob sie die Zelle enthalten
///   2. Unter den zutreffenden Zonen die mit höchster Priorität wählen
///   3. Falls keine Zone zutrifft: Standard-Regel verwenden
pub struct ZoneManager {
    /// Alle registrierten Zonen
    zones: Vec<Zone>,

    /// Standard-Regel wenn keine Zone zutrifft
    default_rule_set: RuleSet,
}

impl ZoneManager {
    // =========================================================================
    // Konstruktor
    // =========================================================================

    /// Erstellt einen neuen ZoneManager mit einer Standard-Regel.
    pub fn new(default_rule_set: RuleSet) -> Self {
        ZoneManager {
            zones: Vec::new(),
            default_rule_set,
        }
    }

    // =========================================================================
    // Zonen verwalten
    // =========================================================================

    /// Fügt eine neue Zone hinzu und gibt ihre ID zurück (Index in der Liste).
    pub fn add_zone(&mut self, zone: Zone) -> usize {
        self.zones.push(zone);
        self.zones.len() - 1
    }

    /// Entfernt eine Zone anhand ihrer ID.
    /// Gibt true zurück wenn die Zone gefunden und entfernt wurde.
    pub fn remove_zone(&mut self, id: usize) -> bool {
        if id < self.zones.len() {
            self.zones.remove(id);
            true
        } else {
            false
        }
    }

    /// Gibt eine Referenz auf eine Zone zurück.
    pub fn get_zone(&self, id: usize) -> Option<&Zone> {
        self.zones.get(id)
    }

    /// Gibt eine veränderliche Referenz auf eine Zone zurück.
    pub fn get_zone_mut(&mut self, id: usize) -> Option<&mut Zone> {
        self.zones.get_mut(id)
    }

    /// Gibt alle Zonen zurück.
    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    /// Ändert die Standard-Regel.
    pub fn set_default_rule(&mut self, rule_set: RuleSet) {
        self.default_rule_set = rule_set;
    }

    /// Gibt die Standard-Regel zurück.
    pub fn default_rule_set(&self) -> &RuleSet {
        &self.default_rule_set
    }

    // =========================================================================
    // Regel für eine Zelle bestimmen
    // =========================================================================

    /// Gibt das RuleSet zurück das für die Zelle an (x, y) gilt.
    ///
    /// Algorithmus:
    ///   1. Alle aktiven Zonen filtern die (x, y) enthalten
    ///   2. Die Zone mit der niedrigsten Prioritätszahl wählen
    ///   3. Falls keine → Standard-Regel
    pub fn rule_set_at(&self, x: isize, y: isize) -> &RuleSet {
        self.zones
            .iter()
            // Nur aktive Zonen die diese Koordinate enthalten
            .filter(|z| z.active && z.contains(x, y))
            // Niedrigste Prioritätszahl = höchste Priorität
            .min_by_key(|z| z.priority)
            // Name der Regel, oder Standard wenn keine Zone passt
            .map(|z| &z.rule_set)
            .unwrap_or(&self.default_rule_set)
    }

    /// Berechnet den nächsten Zustand einer Zelle unter Berücksichtigung
    /// der zuständigen Zone.
    ///
    /// Das ist die zentrale Methode die vom Simulator aufgerufen wird
    /// wenn Zonen aktiv sind.
    pub fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
        let rule_set = self.rule_set_at(x, y);
        let rule: Box<dyn Rule> = rule_set.to_rule();
        rule.next_state(grid, x, y)
    }

    /// Gibt zurück wie viele Zonen aktiv sind.
    pub fn active_zone_count(&self) -> usize {
        self.zones.iter().filter(|z| z.active).count()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_contains() {
        // Zone von (10,10) bis (20,20)
        let zone = Zone::new("test", 10, 10, 10, 10, RuleSet::GameOfLife, 0);

        assert!(zone.contains(10, 10));  // Ecke oben links
        assert!(zone.contains(19, 19)); // Ecke unten rechts
        assert!(zone.contains(15, 15)); // Mitte
        assert!(!zone.contains(9, 10));  // Links außerhalb
        assert!(!zone.contains(20, 10)); // Rechts außerhalb (exclusive)
        assert!(!zone.contains(10, 20)); // Unten außerhalb (exclusive)
    }

    #[test]
    fn test_default_rule_when_no_zones() {
        let manager = ZoneManager::new(RuleSet::GameOfLife);
        // Ohne Zonen gilt überall die Standard-Regel
        assert_eq!(manager.rule_set_at(0, 0), &RuleSet::GameOfLife);
        assert_eq!(manager.rule_set_at(50, 50), &RuleSet::GameOfLife);
    }

    #[test]
    fn test_zone_overrides_default() {
        let mut manager = ZoneManager::new(RuleSet::GameOfLife);

        // Maze-Zone in der Mitte
        manager.add_zone(Zone::new(
            "maze_zone", 10, 10, 20, 20,
            RuleSet::Maze, 0,
        ));

        // Innerhalb der Zone gilt Maze
        assert_eq!(manager.rule_set_at(15, 15), &RuleSet::Maze);
        // Außerhalb gilt weiter GoL
        assert_eq!(manager.rule_set_at(5, 5), &RuleSet::GameOfLife);
    }

    #[test]
    fn test_priority_with_overlapping_zones() {
        let mut manager = ZoneManager::new(RuleSet::GameOfLife);

        // Zone 1: (0,0) bis (19,19) → Maze, niedrige Priorität
        manager.add_zone(Zone::new(
            "low_priority", 0, 0, 20, 20,
            RuleSet::Maze, 10,
        ));
        // Zone 2: (5,5) bis (14,14) → Seeds, hohe Priorität
        manager.add_zone(Zone::new(
            "high_priority", 5, 5, 10, 10,
            RuleSet::Seeds, 1,
        ));

        // Im Überlappungsbereich (beide Zonen) gewinnt Seeds (Priorität 1)
        assert_eq!(manager.rule_set_at(10, 10), &RuleSet::Seeds);

        // Nur in Zone 1 (außerhalb Zone 2) → Maze
        assert_eq!(manager.rule_set_at(2, 2), &RuleSet::Maze);

        // Außerhalb beider Zonen → GoL
        assert_eq!(manager.rule_set_at(25, 25), &RuleSet::GameOfLife);
    }

    #[test]
    fn test_inactive_zone_ignored() {
        let mut manager = ZoneManager::new(RuleSet::GameOfLife);

        let id = manager.add_zone(Zone::new(
            "inactive", 0, 0, 50, 50,
            RuleSet::Maze, 0,
        ));

        // Zone deaktivieren
        manager.get_zone_mut(id).unwrap().active = false;

        // Deaktivierte Zone wird ignoriert → Standard-Regel gilt
        assert_eq!(manager.rule_set_at(25, 25), &RuleSet::GameOfLife);
    }

    #[test]
    fn test_remove_zone() {
        let mut manager = ZoneManager::new(RuleSet::GameOfLife);

        let id = manager.add_zone(Zone::new(
            "to_remove", 0, 0, 50, 50,
            RuleSet::Maze, 0,
        ));

        assert_eq!(manager.rule_set_at(25, 25), &RuleSet::Maze);

        manager.remove_zone(id);

        // Nach Entfernen gilt wieder Standard
        assert_eq!(manager.rule_set_at(25, 25), &RuleSet::GameOfLife);
    }

    #[test]
    fn test_active_zone_count() {
        let mut manager = ZoneManager::new(RuleSet::GameOfLife);

        let id1 = manager.add_zone(Zone::new("z1", 0, 0, 10, 10, RuleSet::Maze, 0));
        let _id2 = manager.add_zone(Zone::new("z2", 20, 20, 10, 10, RuleSet::Seeds, 0));

        assert_eq!(manager.active_zone_count(), 2);

        manager.get_zone_mut(id1).unwrap().active = false;
        assert_eq!(manager.active_zone_count(), 1);
    }
}