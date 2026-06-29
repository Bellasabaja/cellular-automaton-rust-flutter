🧬 Cellular Automaton
Ein erweiterbarer Zellulärer Automat – gebaut mit Rust (Core-Logik) und Flutter Web (Frontend).
Demo
![Cellular Automaton](docs/screenshot.png)Simulation läuft live im Browser
Verschiedene Regeln wechselbar
Zellen per Klick togglen
Geschwindigkeit per Slider steuerbar
***Stack
Schicht Technologie
Core-Logik Rust (ca_core)
Web-Bridge wasm-bindgen + wasm-pack
Frontend Flutter Web
State Management Riverpod
Rendering CustomPainter
***Architektur
┌──────────────────────────────────────┐
│ Flutter Frontend │
│ CustomPainter │ Controls │ Riverpod │
└─────────────────┬────────────────────┘
│ wasm-bindgen (JS Interop)
┌─────────────────▼────────────────────┐
│ Rust Core (ca_core) │
│ Grid │ RuleEngine │ ZoneManager │
│ Simulator │ Config │ Exporter │
└──────────────────────────────────────┘
***Features
Regeln
Regel Notation Beschreibung
Game of Life B3/S23 Klassische Conway-Regel
HighLife B36/S23 Selbst-replizierende Muster
Maze B3/S12345 Labyrinth-generierende Strukturen
Seeds B2/S Explosive chaotische Wachstumsregel
Architektur-Highlights
Rule Trait – neue Regeln ohne Änderung bestehenden Codes
ZoneManager – verschiedene Regeln in verschiedenen Gitterbereichen
Dynamisches Grid – Gittergröße zur Laufzeit änderbar
Wrap-Around – torusförmiges Gitter (Ränder verbunden)
Export – JSON, CSV, PNG
***Projekt-Struktur
cellular-automaton/
├── ca_core/ # Rust Library
│ ├── Cargo.toml
│ └── src/
│ ├── lib.rs
│ ├── api.rs # WASM-bindgen Bridge
│ ├── grid.rs # Grid-Datenstruktur
│ ├── simulator.rs # Simulations-Loop
│ ├── zone.rs # Zonen-Verwaltung
│ ├── config.rs # TOML-Konfiguration
│ ├── rules/
│ │ ├── mod.rs # Rule Trait
│ │ ├── game_of_life.rs
│ │ ├── high_life.rs
│ │ ├── maze.rs
│ │ └── seeds.rs
│ └── export/
│ ├── json.rs
│ ├── csv.rs
│ └── png.rs
└── ca_app/ # Flutter App
├── pubspec.yaml
├── web/
│ ├── index.html # WASM Loading
│ └── pkg/ # Generierte WASM-Dateien
└── lib/
├── main.dart
├── state/
│ └── simulation_state.dart
└── widgets/
└── simulation_screen.dart
\*\*\*Setup & Ausführen
Voraussetzungen

# Rust

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
rustup component add rust-src --toolchain nightly-aarch64-apple-darwin

# wasm-pack

cargo install wasm-pack

# Flutter

# https://flutter.dev/docs/get-started/install

WASM bauen
wasm-pack build ca_core \
 --target no-modules \
 --out-dir ca_app/web/pkg \
 --out-name ca_core
App starten
cd ca_app
flutter run -d chrome \
 --web-header "Cross-Origin-Opener-Policy=same-origin" \
 --web-header "Cross-Origin-Embedder-Policy=require-corp"
***Tests
cd ca_core
cargo test
running 60 tests
test result: ok. 60 passed; 0 failed
Test-Abdeckung
Modul Tests
grid.rs 7
game_of_life.rs 5
simulator.rs 7
high_life.rs 2
maze.rs 3
seeds.rs 3
zone.rs 7
config.rs 9
export/json.rs 4
export/csv.rs 6
export/png.rs 6
Doc-Tests 1
***Neue Regel hinzufügen
Neue Datei anlegen: ca_core/src/rules/meine_regel.rs
use crate::grid::{CellState, Grid};
use super::Rule;
pub struct MeineRegel;
impl Rule for MeineRegel {
fn next_state(&self, grid: &Grid, x: isize, y: isize) -> CellState {
let neighbors = grid.count_alive_neighbors(x, y);
// Eigene Logik hier
if neighbors == 3 { CellState::Alive } else { CellState::Dead }
}
fn name(&self) -> &str { "Meine Regel" }
fn description(&self) -> &str { "Beschreibung" }
}
In rules/mod.rs eintragen:
pub mod meine_regel;
pub enum RuleSet {
// ...
MeineRegel,
}
In api.rs in window.set_rule verfügbar machen – fertig!
***Gelernte Konzepte
Rust
Traits & Generics
Ownership & Borrowing
Result<T, E> Fehlerbehandlung
thread_local! + RefCell für WASM
wasm-bindgen FFI
Unit Tests & Doc-Tests
Flutter
CustomPainter für performantes Rendering
Riverpod State Management
dart:js_interop für WASM-Aufruf
addPostFrameCallback für Widget Lifecycle
***Bekannte Einschränkungen
flutter_rust_bridge v2 hat Web-Threading-Probleme → direktes wasm-bindgen verwendet
GIF-Export noch nicht implementiert
ZoneEditor Widget noch nicht implementiert
