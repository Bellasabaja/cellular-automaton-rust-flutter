

pub mod grid;        // Das Gitter (2D-Array der Zellen)
pub mod simulator;   // Der Simulations-Loop (tick, step, reset)
pub mod zone;        // Zonen-Verwaltung (verschiedene Regeln pro Bereich)
pub mod config;      // Konfiguration laden (Gittergröße, Startregeln etc.)
pub mod rules;       // Alle Regelimplementierungen
pub mod export;      // Exporter (JSON, PNG, CSV)

// Die Bridge – nach außen sichtbar für Flutter
pub mod api;

