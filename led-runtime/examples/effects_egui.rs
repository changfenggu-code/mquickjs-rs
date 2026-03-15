/// LED effects GUI demo — egui version
///
/// Runs all 4 LED effects through the mquickjs JS engine, pre-captures frames,
/// then plays them back as an animated LED strip in an egui window.
///
/// Usage:  cargo run --example effects_egui
use eframe::egui;
use std::time::Duration;

#[path = "common/effects.rs"]
mod effects_common;
use effects_common::EffectData;

const NUM_FRAMES: usize = 120;

// ── egui App ──

struct LedApp {
    effects: Vec<EffectData>,
    current: usize,
    frame_idx: usize,
    last_tick: f64,
    speed_ms: f64,
    paused: bool,
}

impl LedApp {
    fn new() -> Self {
        Self {
            effects: effects_common::capture_all(NUM_FRAMES),
            current: 0,
            frame_idx: 0,
            last_tick: 0.0,
            speed_ms: 80.0,
            paused: false,
        }
    }
}

impl eframe::App for LedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));

        // Advance frame on timer
        let now = ctx.input(|i| i.time);
        if !self.paused && now - self.last_tick >= self.speed_ms / 1000.0 {
            self.last_tick = now;
            let n = self.effects[self.current].frames.len();
            if n > 0 {
                self.frame_idx = (self.frame_idx + 1) % n;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.heading("mquickjs LED Effects");
                ui.add_space(16.0);

                // Effect selector
                ui.horizontal(|ui| {
                    for (i, ef) in self.effects.iter().enumerate() {
                        if ui
                            .add(egui::Button::new(ef.name).selected(i == self.current))
                            .clicked()
                            && i != self.current
                        {
                            self.current = i;
                            self.frame_idx = 0;
                            self.last_tick = now;
                        }
                    }
                });
                ui.add_space(20.0);

                // LED strip
                let ef = &self.effects[self.current];
                if let Some(frame) = ef.frames.get(self.frame_idx) {
                    let sz = 28.0_f32;
                    let gap = 3.0_f32;
                    let total_w = ef.led_count as f32 * (sz + gap) - gap;
                    let (resp, painter) =
                        ui.allocate_painter(egui::vec2(total_w, sz + 8.0), egui::Sense::hover());
                    let org = resp.rect.left_top() + egui::vec2(0.0, 4.0);

                    for (i, rgb) in frame.iter().enumerate() {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(org.x + i as f32 * (sz + gap), org.y),
                            egui::vec2(sz, sz),
                        );
                        let color = if rgb == &[0, 0, 0] {
                            egui::Color32::from_rgb(25, 25, 35)
                        } else {
                            egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                        };
                        painter.rect_filled(rect, 4.0, color);
                    }
                }

                ui.add_space(20.0);

                // Controls
                ui.horizontal(|ui| {
                    if ui
                        .button(if self.paused { "▶ Play" } else { "⏸ Pause" })
                        .clicked()
                    {
                        self.paused = !self.paused;
                    }
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut self.speed_ms, 20.0..=300.0).suffix(" ms"));
                });

                ui.add_space(8.0);
                let ef = &self.effects[self.current];
                ui.label(format!(
                    "Frame {}/{} · {} LEDs",
                    self.frame_idx + 1,
                    ef.frames.len(),
                    ef.led_count
                ));
            });
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "mquickjs LED Effects",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([720.0, 280.0])
                .with_title("mquickjs LED Effects — egui"),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(LedApp::new()))),
    )
}
