// simulator.rs – Der Simulations-Motor
//
// Der Simulator verbindet Grid und Rule.
// Er verwaltet den aktuellen Zustand und berechnet Zeitschritte.
//
// Lernkonzepte:
//   - Structs mit mehreren Feldern
//   - Box<dyn Trait> verwenden
//   - Owned vs Borrowed (Grid gehört dem Simulator)
//   - Vec als History-Speicher
//   - pub/privat Sichtbarkeit

use crate::grid::{CellState, Grid, NeighborhoodType};
use crate::rules::{Rule, RuleSet};

// =============================================================================
// SimulatorConfig – Konfiguration beim Start
// =============================================================================

/// Alle Parameter die beim Erstellen eines Simulators übergeben werden.
///
/// Wir bündeln sie in einem Struct statt viele Einzelparameter zu übergeben.
/// Das macht die API stabiler – neue Parameter brechen keine bestehenden Aufrufe.
pub struct SimulatorConfig {
    /// Breite des Gitters
    pub width: usize,
    /// Höhe des Gitters
    pub height: usize,
    /// Ob Ränder verbunden sind
    pub wrap_around: bool,
    /// Welche Nachbarschaft verwendet wird
    pub neighborhood: NeighborhoodType,
    /// Welche Regel verwendet wird
    pub rule_set: RuleSet,
    /// Wie viele vergangene Generationen gespeichert werden (0 = keine History)
    pub history_size: usize,
    /// Zufällige Startbelegung: None = leeres Grid, Some(density, seed) = zufällig
    pub random_start: Option<(f64, u64)>,
}

impl SimulatorConfig {
    /// Erstellt eine Standard-Konfiguration (100x100, GoL, kein Zufall).
    pub fn default() -> Self {
        SimulatorConfig {
            width: 100,
            height: 100,
            wrap_around: true,
            neighborhood: NeighborhoodType::Moore,
            rule_set: RuleSet::GameOfLife,
            history_size: 0,
            random_start: None,
        }
    }
}

// =============================================================================
// SimulationStats – Statistiken pro Generation
// =============================================================================

/// Statistiken einer Generation – wird in der History gespeichert.
#[derive(Clone, Debug)]
pub struct GenerationStats {
    /// Generationsnummer (0 = Startbelegung)
    pub generation: u64,
    /// Anzahl lebendiger Zellen
    pub alive_count: usize,
    /// Differenz zur Vorgänger-Generation (positiv = Wachstum)
    pub delta: i64,
}

// =============================================================================
// Simulator
// =============================================================================

/// Der Kern-Simulator.
///
/// Er besitzt:
///   - Das aktuelle Grid
///   - Die aktive Regel
///   - Einen Zähler für vergangene Generationen
///   - Eine optionale History vergangener Zustände
pub struct Simulator {
    /// Das aktuelle Gitter (owned – der Simulator ist der einzige Besitzer)
    grid: Grid,

    /// Die aktive Regel als Trait Object
    /// `Box<dyn Rule>` = Heap-allokiertes Objekt mit dynamischem Dispatch
    rule: Box<dyn Rule>,

    /// Aktuelle Generationsnummer
    generation: u64,

    /// Maximale Anzahl gespeicherter History-Einträge
    history_size: usize,

    /// Statistiken vergangener Generationen
    stats_history: Vec<GenerationStats>,

    /// Ob die Simulation läuft
    running: bool,
}

impl Simulator {
    // =========================================================================
    // Konstruktor
    // =========================================================================

    /// Erstellt einen neuen Simulator mit der gegebenen Konfiguration.
    pub fn new(config: SimulatorConfig) -> Self {
        // Grid erstellen – entweder leer oder zufällig befüllt
        let grid = match config.random_start {
            Some((density, seed)) => Grid::new_random(
                config.width,
                config.height,
                config.wrap_around,
                config.neighborhood,
                density,
                seed,
            ),
            None => Grid::new(
                config.width,
                config.height,
                config.wrap_around,
                config.neighborhood,
            ),
        };

        Simulator {
            grid,
            rule: config.rule_set.to_rule(),
            generation: 0,
            history_size: config.history_size,
            stats_history: Vec::new(),
            running: false,
        }
    }

    // =========================================================================
    // Simulation steuern
    // =========================================================================

    /// Startet die Simulation (setzt `running` auf true).
    /// Der eigentliche Loop liegt beim Aufrufer (Flutter / CLI).
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Pausiert die Simulation.
    pub fn pause(&mut self) {
        self.running = false;
    }

    /// Gibt zurück ob die Simulation gerade läuft.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Berechnet einen einzelnen Zeitschritt.
    ///
    /// Das ist die wichtigste Methode des Simulators.
    ///
    /// Ablauf:
    ///   1. Für jede Zelle den neuen Zustand berechnen (liest altes Grid)
    ///   2. Neues Grid übernehmen
    ///   3. Generationszähler erhöhen
    ///   4. Statistiken speichern
    pub fn tick(&mut self) {
        // Wir brauchen zwei Grids: das aktuelle (zum Lesen) und
        // das neue (zum Schreiben). Direkte In-Place-Modifikation würde
        // die Berechnung verfälschen, da spätere Zellen das schon
        // veränderte Grid lesen würden.
        //
        // Lösung: neues Grid als Kopie erstellen, dann befüllen.
        let mut next_grid = self.grid.clone();

        // Über alle Zellen iterieren
        for y in 0..self.grid.height as isize {
            for x in 0..self.grid.width as isize {
                // Neuen Zustand via Regel berechnen (liest self.grid, nicht next_grid)
                let new_state = self.rule.next_state(&self.grid, x, y);
                next_grid.set(x, y, new_state);
            }
        }

        // Statistiken vor dem Tausch berechnen
        let prev_alive = self.grid.count_alive();
        let next_alive = next_grid.count_alive();

        // Altes Grid gegen neues austauschen
        self.grid = next_grid;
        self.generation += 1;

        // Statistiken speichern (wenn history_size > 0)
        if self.history_size > 0 {
            let delta = next_alive as i64 - prev_alive as i64;
            self.stats_history.push(GenerationStats {
                generation: self.generation,
                alive_count: next_alive,
                delta,
            });

            // History-Größe begrenzen (älteste Einträge entfernen)
            if self.stats_history.len() > self.history_size {
                self.stats_history.remove(0);
            }
        }
    }

    /// Führt genau N Zeitschritte aus.
    /// Praktisch für Tests und den "Step"-Button im Frontend.
    pub fn step(&mut self, n: usize) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Setzt die Simulation auf den Ausgangszustand zurück.
    /// Grid wird geleert, Generationszähler auf 0.
    pub fn reset(&mut self) {
        self.grid.clear();
        self.generation = 0;
        self.stats_history.clear();
        self.running = false;
    }

    /// Setzt die Simulation zurück und befüllt das Grid zufällig.
    pub fn reset_random(&mut self, density: f64, seed: u64) {
        let w = self.grid.width;
        let h = self.grid.height;
        let wrap = self.grid.wrap_around;
        let nb = self.grid.neighborhood;

        self.grid = Grid::new_random(w, h, wrap, nb, density, seed);
        self.generation = 0;
        self.stats_history.clear();
        self.running = false;
    }

    // =========================================================================
    // Regel wechseln
    // =========================================================================

    /// Wechselt die aktive Regel.
    /// Das Grid bleibt unverändert – nur die Regel ändert sich.
    pub fn set_rule(&mut self, rule_set: RuleSet) {
        self.rule = rule_set.to_rule();
    }

    // =========================================================================
    // Gittergröße ändern
    // =========================================================================

    /// Ändert die Gittergröße.
    /// Bestehende Zellen werden so gut wie möglich übernommen (via Grid::resize).
    pub fn resize(&mut self, width: usize, height: usize) {
        self.grid.resize(width, height);
    }

    // =========================================================================
    // Lesender Zugriff (für Frontend und Exporter)
    // =========================================================================

    /// Gibt eine Referenz auf das aktuelle Grid zurück.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Gibt eine veränderliche Referenz auf das Grid zurück.
    /// Nützlich für direkte Zell-Manipulationen (z. B. Zeichnen im Frontend).
    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Gibt die aktuelle Generationsnummer zurück.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Gibt die Statistik-History zurück.
    pub fn stats_history(&self) -> &[GenerationStats] {
        &self.stats_history
    }

    /// Gibt die aktuellen Statistiken zurück (ohne History zu benötigen).
    pub fn current_stats(&self) -> GenerationStats {
        GenerationStats {
            generation: self.generation,
            alive_count: self.grid.count_alive(),
            delta: 0, // Keine Vorgänger-Info ohne History
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Hilfsfunktion: Erstellt einen einfachen Simulator (5x5, GoL)
    fn make_simulator() -> Simulator {
        Simulator::new(SimulatorConfig {
            width: 5,
            height: 5,
            wrap_around: false,
            neighborhood: NeighborhoodType::Moore,
            rule_set: RuleSet::GameOfLife,
            history_size: 10,
            random_start: None,
        })
    }

    #[test]
    fn test_initial_state() {
        let sim = make_simulator();
        assert_eq!(sim.generation(), 0);
        assert_eq!(sim.grid().count_alive(), 0);
        assert!(!sim.is_running());
    }

    #[test]
    fn test_start_pause() {
        let mut sim = make_simulator();
        sim.start();
        assert!(sim.is_running());
        sim.pause();
        assert!(!sim.is_running());
    }

    #[test]
    fn test_tick_increments_generation() {
        let mut sim = make_simulator();
        sim.tick();
        assert_eq!(sim.generation(), 1);
        sim.tick();
        assert_eq!(sim.generation(), 2);
    }

    #[test]
    fn test_blinker_oscillates() {
        // Blinker: 3 vertikale Zellen → nach 2 Ticks wieder vertikal
        let mut sim = make_simulator();

        // Vertikalen Blinker setzen
        sim.grid_mut().set(2, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.grid_mut().set(2, 3, CellState::Alive);

        // Nach 1 Tick: horizontal
        sim.tick();
        assert_eq!(sim.grid().get(1, 2), CellState::Alive);
        assert_eq!(sim.grid().get(2, 2), CellState::Alive);
        assert_eq!(sim.grid().get(3, 2), CellState::Alive);
        assert_eq!(sim.grid().get(2, 1), CellState::Dead);
        assert_eq!(sim.grid().get(2, 3), CellState::Dead);

        // Nach 2 Ticks: wieder vertikal
        sim.tick();
        assert_eq!(sim.grid().get(2, 1), CellState::Alive);
        assert_eq!(sim.grid().get(2, 2), CellState::Alive);
        assert_eq!(sim.grid().get(2, 3), CellState::Alive);
    }

    #[test]
    fn test_reset_clears_grid() {
        let mut sim = make_simulator();
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.tick();
        sim.reset();

        assert_eq!(sim.generation(), 0);
        assert_eq!(sim.grid().count_alive(), 0);
        assert!(!sim.is_running());
    }

    #[test]
    fn test_step_multiple() {
        let mut sim = make_simulator();
        sim.step(5);
        assert_eq!(sim.generation(), 5);
    }

    #[test]
    fn test_stats_history() {
        let mut sim = make_simulator();

        // Blinker setzen (3 lebende Zellen)
        sim.grid_mut().set(2, 1, CellState::Alive);
        sim.grid_mut().set(2, 2, CellState::Alive);
        sim.grid_mut().set(2, 3, CellState::Alive);

        sim.tick();

        let history = sim.stats_history();
        assert_eq!(history.len(), 1);
        // Blinker hat immer 3 lebende Zellen (ändert nur Form)
        assert_eq!(history[0].alive_count, 3);
        assert_eq!(history[0].generation, 1);
    }
}