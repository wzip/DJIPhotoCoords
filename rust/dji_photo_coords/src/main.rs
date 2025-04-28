//William Zipse 04/2025
//with AI assistance
// src/main.rs

use anyhow::Result;
use eframe::egui;
use exif::{In, Tag, Reader, Value};
use std::path::Path;
use walkdir::WalkDir;

struct DJIPhotoCoordsApp {
    input_folder: String,
    output_file: String,
    status: String,
    show_license: bool,
    license_text: String,
}

impl Default for DJIPhotoCoordsApp {
    fn default() -> Self {
        let license_text = match std::fs::read_to_string("LICENSE.txt") {
            Ok(text) => text,
            Err(_) => {
                "MIT License

Copyright (c) 2022 William E. Zipse

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.".to_string()
            }
        };

        Self {
            input_folder: String::new(),
            output_file: String::new(),
            status: String::new(),
            show_license: false,
            license_text,
        }
    }
}

impl eframe::App for DJIPhotoCoordsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_license = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Show license popup if needed
        if self.show_license {
            egui::Window::new("License")
                .open(&mut self.show_license)
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("MIT License");
                        ui.separator();
                        ui.add_space(8.0);
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.label(&self.license_text);
                            });
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DJI Photo Coordinates Extractor");

            // Input Folder
            ui.horizontal(|ui| {
                ui.label("Input Folder:");
                ui.text_edit_singleline(&mut self.input_folder);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.input_folder = path.to_string_lossy().to_string();
                    }
                }
            });

            // Output File
            ui.horizontal(|ui| {
                ui.label("Output CSV File:");
                ui.text_edit_singleline(&mut self.output_file);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("CSV", &["csv"])
                        .save_file()
                    {
                        self.output_file = path.to_string_lossy().to_string();
                    }
                }
            });

            // Process Button
            if ui.button("Process Photos").clicked() {
                if let Err(e) = self.process_photos() {
                    self.status = format!("Error: {}", e);
                }
            }

            // Status
            ui.label(&self.status);
        });
    }
}

impl DJIPhotoCoordsApp {
    fn process_photos(&mut self) -> Result<()> {
        let input_path = Path::new(&self.input_folder);
        let output_path = Path::new(&self.output_file);

        if !input_path.exists() {
            return Err(anyhow::anyhow!("Input folder does not exist"));
        }

        let mut wtr = csv::Writer::from_path(output_path)?;
        wtr.write_record(&[
            "FileName",
            "Latitude_DMS",
            "Longitude_DMS",
            "Latitude_DD",
            "Longitude_DD",
            "Altitude_m",
            "Altitude_ft",
        ])?;

        let mut processed_count = 0;
        for entry in WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("jpg"))
            })
        {
            let path = entry.path();
            if let Ok(record) = self.process_image(path) {
                wtr.write_record(&record)?;
                processed_count += 1;
            }
        }

        wtr.flush()?;
        self.status = format!("Processed {} photos", processed_count);
        Ok(())
    }

    fn process_image(&self, path: &Path) -> Result<Vec<String>> {
        // Read EXIF
        let file = std::fs::File::open(path)?;
        let mut buf = std::io::BufReader::new(file);
        let exif_reader = Reader::new();
        let exif = exif_reader.read_from_container(&mut buf)?;

        // Start CSV row
        let mut row = Vec::new();
        row.push(path.file_name().unwrap().to_string_lossy().into_owned());

        // Fetch GPS fields or error
        let lat_field = exif
            .get_field(Tag::GPSLatitude, In::PRIMARY)
            .ok_or_else(|| anyhow::anyhow!("No GPS latitude"))?;
        let lat_ref_field = exif
            .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
            .ok_or_else(|| anyhow::anyhow!("No GPS latitude ref"))?;
        let lon_field = exif
            .get_field(Tag::GPSLongitude, In::PRIMARY)
            .ok_or_else(|| anyhow::anyhow!("No GPS longitude"))?;
        let lon_ref_field = exif
            .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
            .ok_or_else(|| anyhow::anyhow!("No GPS longitude ref"))?;
        let alt_field = exif
            .get_field(Tag::GPSAltitude, In::PRIMARY)
            .ok_or_else(|| anyhow::anyhow!("No GPS altitude"))?;

        // Convert DMS → decimal degrees
        let lat = Self::rational_to_degrees(&lat_field.value)?;
        let lon = Self::rational_to_degrees(&lon_field.value)?;

        // Extract "N" / "S"
        let lat_ref = match &lat_ref_field.value {
            Value::Ascii(vec) if !vec.is_empty() => {
                String::from_utf8_lossy(&vec[0]).into_owned()
            }
            _ => return Err(anyhow::anyhow!("Invalid GPS latitude ref")),
        };
        let lon_ref = match &lon_ref_field.value {
            Value::Ascii(vec) if !vec.is_empty() => {
                String::from_utf8_lossy(&vec[0]).into_owned()
            }
            _ => return Err(anyhow::anyhow!("Invalid GPS longitude ref")),
        };

        let lat_dd = if lat_ref == "S" { -lat } else { lat };
        let lon_dd = if lon_ref == "W" { -lon } else { lon };

        // Extract altitude (just one RATIONAL pair)
        let alt_m = match &alt_field.value {
            Value::Rational(vec) if !vec.is_empty() => vec[0].to_f64(),
            _ => return Err(anyhow::anyhow!("Invalid GPS altitude")),
        };
        let alt_ft = alt_m * 3.28084;

        // Format DMS strings
        let lat_dms = Self::decimal_to_dms(lat_dd, &lat_ref);
        let lon_dms = Self::decimal_to_dms(lon_dd, &lon_ref);

        // Push into CSV row
        row.push(lat_dms);
        row.push(lon_dms);
        row.push(format!("{:.6}", lat_dd));
        row.push(format!("{:.6}", lon_dd));
        row.push(format!("{:.2}", alt_m));
        row.push(format!("{:.2}", alt_ft));

        Ok(row)
    }

    fn rational_to_degrees(value: &Value) -> Result<f64> {
        if let Value::Rational(ref v) = *value {
            if v.len() >= 3 {
                // degrees + minutes/60 + seconds/3600
                return Ok(v[0].to_f64() + v[1].to_f64() / 60.0 + v[2].to_f64() / 3600.0);
            }
        }
        Err(anyhow::anyhow!("Invalid rational value")) // matching Value::Rational :contentReference[oaicite:0]{index=0}
    }

    fn decimal_to_dms(decimal: f64, direction: &str) -> String {
        let deg = decimal.abs().trunc();
        let min = ((decimal.abs() - deg) * 60.0).trunc();
        let sec = (decimal.abs() - deg - min / 60.0) * 3600.0;
        format!("{}° {}' {:.2}\" {}", deg, min, sec, direction)
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DJI Photo Coordinates Extractor",
        options,
        Box::new(|_cc| Box::new(DJIPhotoCoordsApp::default())),
    )
    .map_err(|e| anyhow::anyhow!("Application error: {}", e))?;
    Ok(())
}
