// config.rs – Konfiguration laden und validieren
//
// Die Config-Datei ist eine TOML-Datei die beim Start geladen wird.
// Sie definiert Gittergröße, Startregeln, Zonen und mehr.
//
// Lernkonzepte:
//   - serde Deserialisierung
//   - #[derive(Deserialize)]
//   - Default-Werte mit #[serde(default)]
//   - Result<T, E> für Fehlerbehandlung
//   - std::fs für Datei-Operationen

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::grid::NeighborhoodType;
use crate::rules::RuleSet;
use crate::simulator::SimulatorConfig;
use crate::zone::{Zone, ZoneManager};

// =============================================================================
// Config-Strukturen (gespiegeln die TOML-Datei)
// =============================================================================

/// Haupt-Konfiguration – entspricht der TOML-Datei.
///
/// `#[derive(Deserialize)]` lässt serde die TOML-Datei automatisch
/// in diese Struktur umwandeln. Feldnamen müssen mit TOML-Keys übereinstimmen.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Gitter-Einstellungen
    #[serde(default)]
    pub grid: GridConfig,

    /// Simulations-Einstellungen
    #[serde(default)]
    pub simulation: SimulationConfig,

    /// Zonen (optional, kann leer sein)
    #[serde(default)]
    pub zones: Vec<ZoneConfig>,
}

/// Gitter-Konfiguration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GridConfig {
    /// Breite in Zellen
    #[serde(default = "default_width")]
    pub width: usize,

    /// Höhe in Zellen
    #[serde(default = "default_height")]
    pub height: usize,

    /// Ob Ränder verbunden sind
    #[serde(default = "default_true")]
    pub wrap_around: bool,

    /// Nachbarschaftstyp: "moore" oder "von_neumann"
    #[serde(default = "default_neighborhood")]
    pub neighborhood: String,
}

/// Simulations-Konfiguration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationConfig {
    /// Regelname: "game_of_life", "high_life", "maze", "seeds"
    #[serde(default = "default_rule")]
    pub rule: String,

    /// Zufällige Startbelegung
    #[serde(default)]
    pub random_start: bool,

    /// Dichte der zufälligen Startbelegung (0.0 – 1.0)
    #[serde(default = "default_density")]
    pub density: f64,

    /// Seed für den Zufallsgenerator
    #[serde(default = "default_seed")]
    pub seed: u64,

    /// Wie viele Generationen in der History gespeichert werden
    #[serde(default = "default_history_size")]
    pub history_size: usize,
}

/// Konfiguration einer einzelnen Zone
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZoneConfig {
    /// Name der Zone
    pub name: String,
    /// Position und Größe
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
    /// Regel in dieser Zone
    pub rule: String,
    /// Priorität (0 = höchste)
    #[serde(default)]
    pub priority: u32,
}

// =============================================================================
// Default-Werte
// =============================================================================
// serde braucht Funktionen statt Literale für #[serde(default = "...")]

fn default_width()         -> usize  { 100 }
fn default_height()        -> usize  { 100 }
fn default_true()          -> bool   { true }
fn default_neighborhood()  -> String { "moore".to_string() }
fn default_rule()          -> String { "game_of_life".to_string() }
fn default_density()       -> f64    { 0.3 }
fn default_seed()          -> u64    { 42 }
fn default_history_size()  -> usize  { 100 }

// =============================================================================
// Default-Implementierungen für die Config-Structs
// =============================================================================

impl Default for Config {
    fn default() -> Self {
        Config {
            grid: GridConfig::default(),
            simulation: SimulationConfig::default(),
            zones: Vec::new(),
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig {
            width: default_width(),
            height: default_height(),
            wrap_around: default_true(),
            neighborhood: default_neighborhood(),
        }
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            rule: default_rule(),
            random_start: false,
            density: default_density(),
            seed: default_seed(),
            history_size: default_history_size(),
        }
    }
}

// =============================================================================
// Config laden und parsen
// =============================================================================

/// Alle möglichen Fehler beim Config-Laden.
///
/// In Rust ist es best practice, eigene Fehlertypen zu definieren
/// statt generische Strings zu verwenden.
#[derive(Debug)]
pub enum ConfigError {
    /// Datei konnte nicht gelesen werden
    IoError(std::io::Error),
    /// TOML konnte nicht geparst werden
    ParseError(toml::de::Error),
    /// Ein Feldwert ist ungültig
    InvalidValue(String),
}

/// Implementierung von Display für schöne Fehlermeldungen
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e)       => write!(f, "Datei-Fehler: {}", e),
            ConfigError::ParseError(e)    => write!(f, "TOML-Fehler: {}", e),
            ConfigError::InvalidValue(s)  => write!(f, "Ungültiger Wert: {}", s),
        }
    }
}

// `From` Traits erlauben automatische Konvertierung mit `?` Operator
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IoError(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::ParseError(e)
    }
}

/// Lädt eine Config-Datei von einem Pfad.
///
/// # Fehlerbehandlung mit `?`
/// Der `?` Operator ist Rusts elegante Art mit Fehlern umzugehen:
/// Falls ein Fehler auftritt, wird er sofort zurückgegeben.
/// `fs::read_to_string(path)?` = "lies die Datei, oder gib den Fehler zurück"
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    // Datei lesen – `?` gibt bei Fehler sofort ConfigError::IoError zurück
    let content = fs::read_to_string(path)?;

    // TOML parsen – `?` gibt bei Fehler sofort ConfigError::ParseError zurück
    let config: Config = toml::from_str(&content)?;

    // Werte validieren
    validate_config(&config)?;

    Ok(config)
}

/// Gibt eine Config mit Standard-Werten zurück.
/// Nützlich wenn keine Datei vorhanden ist.
pub fn default_config() -> Config {
    Config::default()
}

/// Validiert die geladene Konfiguration.
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Gittergröße
    if config.grid.width == 0 || config.grid.height == 0 {
        return Err(ConfigError::InvalidValue(
            "Gittergröße muss größer als 0 sein".to_string()
        ));
    }

    // Dichte
    if config.simulation.density < 0.0 || config.simulation.density > 1.0 {
        return Err(ConfigError::InvalidValue(
            format!("Dichte muss zwischen 0.0 und 1.0 liegen, war: {}",
                config.simulation.density)
        ));
    }

    // Regelname
    parse_rule(&config.simulation.rule)?;

    // Zonen validieren
    for zone in &config.zones {
        if zone.width == 0 || zone.height == 0 {
            return Err(ConfigError::InvalidValue(
                format!("Zone '{}': Größe muss größer als 0 sein", zone.name)
            ));
        }
        parse_rule(&zone.rule)?;
    }

    Ok(())
}

// =============================================================================
// Hilfsfunktionen: String → Enum
// =============================================================================

/// Parst einen Regel-String zu einem RuleSet.
pub fn parse_rule(rule: &str) -> Result<RuleSet, ConfigError> {
    match rule.to_lowercase().as_str() {
        "game_of_life" | "gol"      => Ok(RuleSet::GameOfLife),
        "high_life"    | "highlife" => Ok(RuleSet::HighLife),
        "maze"                      => Ok(RuleSet::Maze),
        "seeds"                     => Ok(RuleSet::Seeds),
        other => Err(ConfigError::InvalidValue(
            format!("Unbekannte Regel: '{}'. Gültig: game_of_life, high_life, maze, seeds", other)
        )),
    }
}

/// Parst einen Nachbarschafts-String zu einem NeighborhoodType.
pub fn parse_neighborhood(nb: &str) -> Result<NeighborhoodType, ConfigError> {
    match nb.to_lowercase().as_str() {
        "moore"                          => Ok(NeighborhoodType::Moore),
        "von_neumann" | "vonneumann"     => Ok(NeighborhoodType::VonNeumann),
        other => Err(ConfigError::InvalidValue(
            format!("Unbekannte Nachbarschaft: '{}'. Gültig: moore, von_neumann", other)
        )),
    }
}

// =============================================================================
// Config → SimulatorConfig + ZoneManager
// =============================================================================

/// Konvertiert eine Config in einen SimulatorConfig und einen ZoneManager.
///
/// Das ist der Haupteinstiegspunkt: Config laden → Simulator erstellen.
pub fn config_to_simulator(config: &Config)
    -> Result<(SimulatorConfig, ZoneManager), ConfigError>
{
    let neighborhood = parse_neighborhood(&config.grid.neighborhood)?;
    let rule_set     = parse_rule(&config.simulation.rule)?;

    let random_start = if config.simulation.random_start {
        Some((config.simulation.density, config.simulation.seed))
    } else {
        None
    };

    let sim_config = SimulatorConfig {
        width:         config.grid.width,
        height:        config.grid.height,
        wrap_around:   config.grid.wrap_around,
        neighborhood,
        rule_set:      rule_set.clone(),
        history_size:  config.simulation.history_size,
        random_start,
    };

    // ZoneManager aufbauen
    let mut zone_manager = ZoneManager::new(rule_set);
    for zone_cfg in &config.zones {
        let zone_rule = parse_rule(&zone_cfg.rule)?;
        zone_manager.add_zone(Zone::new(
            &zone_cfg.name,
            zone_cfg.x,
            zone_cfg.y,
            zone_cfg.width,
            zone_cfg.height,
            zone_rule,
            zone_cfg.priority,
        ));
    }

    Ok((sim_config, zone_manager))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = default_config();
        assert!(validate_config(&config).is_ok());
        assert_eq!(config.grid.width, 100);
        assert_eq!(config.grid.height, 100);
        assert!(config.grid.wrap_around);
    }

    #[test]
    fn test_parse_rule_valid() {
        assert_eq!(parse_rule("game_of_life").unwrap(), RuleSet::GameOfLife);
        assert_eq!(parse_rule("gol").unwrap(),          RuleSet::GameOfLife);
        assert_eq!(parse_rule("high_life").unwrap(),    RuleSet::HighLife);
        assert_eq!(parse_rule("maze").unwrap(),         RuleSet::Maze);
        assert_eq!(parse_rule("seeds").unwrap(),        RuleSet::Seeds);
        // Case-insensitive
        assert_eq!(parse_rule("MAZE").unwrap(),         RuleSet::Maze);
    }

    #[test]
    fn test_parse_rule_invalid() {
        let result = parse_rule("unknown_rule");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_neighborhood() {
        assert_eq!(
            parse_neighborhood("moore").unwrap(),
            NeighborhoodType::Moore
        );
        assert_eq!(
            parse_neighborhood("von_neumann").unwrap(),
            NeighborhoodType::VonNeumann
        );
    }

    #[test]
    fn test_invalid_grid_size() {
        let mut config = default_config();
        config.grid.width = 0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_invalid_density() {
        let mut config = default_config();
        config.simulation.density = 1.5; // > 1.0
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_toml_parsing() {
        // TOML direkt als String parsen (kein Dateisystem nötig)
        let toml_str = r#"
            [grid]
            width = 200
            height = 150
            wrap_around = false
            neighborhood = "moore"

            [simulation]
            rule = "maze"
            random_start = true
            density = 0.4
            seed = 123
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.grid.width, 200);
        assert_eq!(config.grid.height, 150);
        assert!(!config.grid.wrap_around);
        assert_eq!(config.simulation.rule, "maze");
        assert_eq!(config.simulation.density, 0.4);
    }

    #[test]
    fn test_config_to_simulator() {
        let config = default_config();
        let result = config_to_simulator(&config);
        assert!(result.is_ok());

        let (sim_config, zone_manager) = result.unwrap();
        assert_eq!(sim_config.width, 100);
        assert_eq!(zone_manager.active_zone_count(), 0);
    }

    #[test]
    fn test_config_with_zones() {
        let toml_str = r#"
            [grid]
            width = 100
            height = 100

            [simulation]
            rule = "game_of_life"

            [[zones]]
            name = "maze_corner"
            x = 0
            y = 0
            width = 20
            height = 20
            rule = "maze"
            priority = 0
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let (_, zone_manager) = config_to_simulator(&config).unwrap();
        assert_eq!(zone_manager.active_zone_count(), 1);
        assert_eq!(zone_manager.rule_set_at(10, 10), &RuleSet::Maze);
        assert_eq!(zone_manager.rule_set_at(50, 50), &RuleSet::GameOfLife);
    }
}