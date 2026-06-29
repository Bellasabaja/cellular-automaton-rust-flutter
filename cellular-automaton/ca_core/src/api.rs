// api.rs – FFI Bridge Funktionen
//
// Alle Funktionen in dieser Datei werden von flutter_rust_bridge
// zu Dart-Funktionen generiert.
//
// Regeln für Bridge-Funktionen:
//   - Nur einfache Typen als Parameter (u32, bool, String, Vec<T>)
//   - Kein borrowed state (&self) – Flutter verwaltet State selbst
//   - Rückgabewerte müssen serialisierbar sein
//
// Lernkonzepte:
//   - FFI (Foreign Function Interface)
//   - Datenstrukturen für den Dart-Transfer
//   - Zustandslos vs. zustandsbehaftet

// api.rs – WASM-bindgen direkte Integration
// Kein flutter_rust_bridge – direkte JS-Bindung

use wasm_bindgen::prelude::*;
use std::cell::RefCell;

use crate::grid::NeighborhoodType;
use crate::rules::RuleSet;
use crate::simulator::{Simulator, SimulatorConfig};
use crate::zone::{Zone, ZoneManager};
use crate::config::{parse_rule, parse_neighborhood};

// Panic Hook für bessere Fehlermeldungen
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Globaler State
thread_local! {
    static SIMULATOR:    RefCell<Option<Simulator>>   = RefCell::new(None);
    static ZONE_MANAGER: RefCell<Option<ZoneManager>> = RefCell::new(None);
}

macro_rules! with_sim {
    ($sim:ident, $body:expr) => {
        SIMULATOR.with(|cell| {
            let mut opt = cell.borrow_mut();
            let $sim = opt.as_mut().expect("Simulator nicht initialisiert");
            $body
        })
    };
}

// Hilfsfunktion: Grid als JSON-String zurückgeben
fn grid_to_json(sim: &Simulator) -> String {
    let grid = sim.grid();
    let cells: Vec<u8> = grid.cells()
        .iter()
        .map(|c| if c.is_alive() { 1 } else { 0 })
        .collect();

    format!(
        r#"{{"width":{},"height":{},"cells":[{}],"generation":{},"aliveCount":{}}}"#,
        grid.width,
        grid.height,
        cells.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","),
        sim.generation(),
        grid.count_alive()
    )
}

#[wasm_bindgen]
pub fn init_simulator(
    width: u32,
    height: u32,
    wrap_around: bool,
    neighborhood: &str,
    rule: &str,
    history_size: u32,
    random_start: bool,
    density: f64,
    seed: u32,
) -> String {
    let nb = parse_neighborhood(neighborhood)
        .unwrap_or(NeighborhoodType::Moore);
    let rule_set = parse_rule(rule)
        .unwrap_or(RuleSet::GameOfLife);

    let random = if random_start {
        Some((density, seed as u64))
    } else {
        None
    };

    let config = SimulatorConfig {
        width:        width as usize,
        height:       height as usize,
        wrap_around,
        neighborhood: nb,
        rule_set:     rule_set.clone(),
        history_size: history_size as usize,
        random_start: random,
    };

    let simulator = Simulator::new(config);
    let json = grid_to_json(&simulator);

    SIMULATOR.with(|cell| { *cell.borrow_mut() = Some(simulator); });
    ZONE_MANAGER.with(|cell| { *cell.borrow_mut() = Some(ZoneManager::new(rule_set)); });

    json
}

#[wasm_bindgen]
pub fn tick() -> String {
    with_sim!(sim, {
        sim.tick();
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn step(n: u32) -> String {
    with_sim!(sim, {
        sim.step(n as usize);
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn reset() -> String {
    with_sim!(sim, {
        sim.reset();
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn toggle_cell(x: i32, y: i32) -> String {
    with_sim!(sim, {
        sim.grid_mut().toggle(x as isize, y as isize);
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn set_rule(rule: &str) -> String {
    with_sim!(sim, {
        let rule_set = parse_rule(rule).unwrap_or(RuleSet::GameOfLife);
        sim.set_rule(rule_set);
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn resize_grid(width: u32, height: u32) -> String {
    with_sim!(sim, {
        sim.resize(width as usize, height as usize);
        grid_to_json(sim)
    })
}

#[wasm_bindgen]
pub fn get_grid_state() -> String {
    with_sim!(sim, { grid_to_json(sim) })
}

#[wasm_bindgen]
pub fn export_json_state() -> String {
    with_sim!(sim, {
        crate::export::json::export_json_string(sim, "game_of_life")
            .unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
    })
}

#[wasm_bindgen]
pub fn export_csv_state() -> String {
    with_sim!(sim, {
        crate::export::csv::export_csv_string(sim)
    })
}