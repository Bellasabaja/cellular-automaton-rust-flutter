// export/png.rs – PNG Export
//
// Rendert das Gitter als PNG-Bild.
// Jede Zelle wird als farbiges Rechteck gezeichnet.
//
// Lernkonzepte:
//   - image crate verwenden
//   - Pixel direkt manipulieren
//   - Farbwerte als RGB
//   - Skalierung (eine Zelle = mehrere Pixel)

use std::path::Path;
use image::{ImageBuffer, Rgb, RgbImage};

use crate::grid::{CellState, Grid};
use crate::simulator::Simulator;

// =============================================================================
// Fehlertyp
// =============================================================================

#[derive(Debug)]
pub enum PngExportError {
    ImageError(image::ImageError),
    InvalidConfig(String),
}

impl std::fmt::Display for PngExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PngExportError::ImageError(e)     => write!(f, "Bild-Fehler: {}", e),
            PngExportError::InvalidConfig(s)  => write!(f, "Konfigurationsfehler: {}", s),
        }
    }
}

impl From<image::ImageError> for PngExportError {
    fn from(e: image::ImageError) -> Self {
        PngExportError::ImageError(e)
    }
}

// =============================================================================
// PngConfig – Aussehen des Exports
// =============================================================================

/// Konfiguration für den PNG-Export.
pub struct PngConfig {
    /// Pixel pro Zelle (1 = minimal, 8 = gut sichtbar)
    pub cell_size: u32,

    /// Farbe lebendiger Zellen (RGB)
    pub alive_color: [u8; 3],

    /// Farbe toter Zellen (RGB)
    pub dead_color: [u8; 3],

    /// Ob ein Gitter gezeichnet werden soll
    pub show_grid: bool,

    /// Farbe des Gitters (RGB) – nur wenn show_grid = true
    pub grid_color: [u8; 3],
}

impl PngConfig {
    /// Standard: schwarze tote Zellen, weiße lebende Zellen, 4px pro Zelle
    pub fn default() -> Self {
        PngConfig {
            cell_size:   4,
            alive_color: [255, 255, 255], // Weiß
            dead_color:  [0,   0,   0  ], // Schwarz
            show_grid:   false,
            grid_color:  [50,  50,  50 ], // Dunkelgrau
        }
    }

    /// Klassisches GoL-Aussehen: grüne Zellen auf schwarz
    pub fn classic_green() -> Self {
        PngConfig {
            cell_size:   4,
            alive_color: [0, 255, 70],  // Grün
            dead_color:  [0, 0,   0 ],  // Schwarz
            show_grid:   false,
            grid_color:  [20, 20, 20],
        }
    }
}

// =============================================================================
// Export
// =============================================================================

/// Exportiert das aktuelle Gitter als PNG-Datei.
pub fn export_png(
    simulator: &Simulator,
    path: &Path,
    config: &PngConfig,
) -> Result<(), PngExportError> {
    let img = render_grid(simulator.grid(), config)?;
    img.save(path)?;
    Ok(())
}

/// Rendert das Gitter in einen ImageBuffer (ohne Datei zu schreiben).
/// Nützlich für Tests und spätere Web-Integration.
pub fn render_grid(
    grid: &Grid,
    config: &PngConfig,
) -> Result<RgbImage, PngExportError> {
    if config.cell_size == 0 {
        return Err(PngExportError::InvalidConfig(
            "cell_size muss größer als 0 sein".to_string()
        ));
    }

    // Bildgröße berechnen
    let img_width  = grid.width  as u32 * config.cell_size;
    let img_height = grid.height as u32 * config.cell_size;

    // ImageBuffer erstellen – jeder Pixel ist ein RGB-Wert
    // ImageBuffer::new() füllt mit Schwarz (0, 0, 0)
    let mut img: RgbImage = ImageBuffer::new(img_width, img_height);

    // Jede Zelle zeichnen
    for y in 0..grid.height {
        for x in 0..grid.width {
            let state = grid.get(x as isize, y as isize);
            let color = match state {
                CellState::Alive => Rgb(config.alive_color),
                CellState::Dead  => Rgb(config.dead_color),
            };

            // Alle Pixel innerhalb der Zelle einfärben
            // Eine Zelle (x, y) belegt Pixel von
            //   (x * cell_size, y * cell_size) bis
            //   ((x+1) * cell_size, (y+1) * cell_size)
            let px_start_x = x as u32 * config.cell_size;
            let px_start_y = y as u32 * config.cell_size;

            for py in px_start_y..px_start_y + config.cell_size {
                for px in px_start_x..px_start_x + config.cell_size {
                    img.put_pixel(px, py, color);
                }
            }
        }
    }

    // Gitter-Linien zeichnen wenn gewünscht
    if config.show_grid {
        draw_grid_lines(&mut img, grid.width, grid.height, config);
    }

    Ok(img)
}

/// Zeichnet Gitter-Linien über das Bild.
fn draw_grid_lines(
    img: &mut RgbImage,
    grid_width: usize,
    grid_height: usize,
    config: &PngConfig,
) {
    let grid_color = Rgb(config.grid_color);
    let cell_size  = config.cell_size;

    // Vertikale Linien
    for x in 0..=grid_width as u32 {
        let px = x * cell_size;
        if px < img.width() {
            for py in 0..img.height() {
                img.put_pixel(px, py, grid_color);
            }
        }
    }

    // Horizontale Linien
    for y in 0..=grid_height as u32 {
        let py = y * cell_size;
        if py < img.height() {
            for px in 0..img.width() {
                img.put_pixel(px, py, grid_color);
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellState, Grid, NeighborhoodType};
    use crate::rules::RuleSet;
    use crate::simulator::{Simulator, SimulatorConfig};

    fn make_simulator(w: usize, h: usize) -> Simulator {
        Simulator::new(SimulatorConfig {
            width: w,
            height: h,
            wrap_around: false,
            neighborhood: NeighborhoodType::Moore,
            rule_set: RuleSet::GameOfLife,
            history_size: 0,
            random_start: None,
        })
    }

    #[test]
    fn test_image_dimensions() {
        let sim    = make_simulator(10, 10);
        let config = PngConfig::default(); // cell_size = 4
        let img    = render_grid(sim.grid(), &config).unwrap();

        // 10 Zellen * 4 Pixel = 40x40
        assert_eq!(img.width(),  40);
        assert_eq!(img.height(), 40);
    }

    #[test]
    fn test_alive_cell_color() {
        let mut sim = make_simulator(5, 5);
        sim.grid_mut().set(0, 0, CellState::Alive);

        let config = PngConfig::default();
        let img    = render_grid(sim.grid(), &config).unwrap();

        // Pixel (0,0) muss alive_color sein (Weiß)
        assert_eq!(img.get_pixel(0, 0), &Rgb([255, 255, 255]));
    }

    #[test]
    fn test_dead_cell_color() {
        let sim    = make_simulator(5, 5);
        let config = PngConfig::default();
        let img    = render_grid(sim.grid(), &config).unwrap();

        // Alle Zellen tot → alle Pixel schwarz
        assert_eq!(img.get_pixel(0, 0), &Rgb([0, 0, 0]));
    }

    #[test]
    fn test_cell_size_scaling() {
        let sim    = make_simulator(10, 10);
        let config = PngConfig {
            cell_size: 8,
            ..PngConfig::default()
        };
        let img = render_grid(sim.grid(), &config).unwrap();

        // 10 * 8 = 80
        assert_eq!(img.width(),  80);
        assert_eq!(img.height(), 80);
    }

    #[test]
    fn test_invalid_cell_size() {
        let sim    = make_simulator(5, 5);
        let config = PngConfig {
            cell_size: 0, // ungültig
            ..PngConfig::default()
        };
        let result = render_grid(sim.grid(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_colors() {
        let mut sim = make_simulator(5, 5);
        sim.grid_mut().set(0, 0, CellState::Alive);

        let config = PngConfig {
            alive_color: [255, 0, 0], // Rot
            dead_color:  [0, 0, 255], // Blau
            ..PngConfig::default()
        };
        let img = render_grid(sim.grid(), &config).unwrap();

        // Lebende Zelle = Rot
        assert_eq!(img.get_pixel(0, 0), &Rgb([255, 0, 0]));
        // Tote Zelle = Blau
        assert_eq!(img.get_pixel(4 * 1, 0), &Rgb([0, 0, 255]));
    }
}