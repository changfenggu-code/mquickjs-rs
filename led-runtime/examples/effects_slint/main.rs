/// LED effects GUI demo — Slint version
///
/// Runs all 4 LED effects through the mquickjs JS engine, pre-captures frames,
/// then plays them back as an animated LED strip in a Slint window.
///
/// Usage:  cargo run --example effects_slint
extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use slint::{Color, SharedString, VecModel};

#[path = "../common/effects.rs"]
mod effects_common;
const NUM_FRAMES: usize = 120;

// ── Slint UI ──

slint::slint! {
    export component MainWindow inherits Window {
        title: "mquickjs LED Effects — Slint";
        preferred-width: 720px;
        preferred-height: 300px;
        background: #1a1a2e;

        in property <[color]> led-colors;
        in property <string> frame-info: "";

        callback select-effect(int);

        VerticalLayout {
            alignment: center;
            padding: 16px;
            spacing: 16px;

            Text {
                text: "mquickjs LED Effects";
                font-size: 20px;
                color: #e0e0e0;
                horizontal-alignment: center;
            }

            HorizontalLayout {
                alignment: center;
                spacing: 8px;

                for item[idx] in [
                    { label: "Blink" },
                    { label: "Chase" },
                    { label: "Rainbow" },
                    { label: "Wave" },
                ]: Rectangle {
                    width: 80px;
                    height: 32px;
                    border-radius: 6px;
                    background: ta.pressed ? #555577 : ta.has-hover ? #444466 : #333355;

                    ta := TouchArea { clicked => { root.select-effect(idx); } }

                    Text {
                        text: item.label;
                        color: white;
                        font-size: 14px;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                }
            }

            HorizontalLayout {
                alignment: center;
                spacing: 3px;

                for c in root.led-colors: Rectangle {
                    width: 28px;
                    height: 28px;
                    border-radius: 4px;
                    background: c;
                }
            }

            Text {
                text: root.frame-info;
                color: #888888;
                font-size: 12px;
                horizontal-alignment: center;
            }
        }
    }
}

fn frame_to_colors(frame: &[[u8; 3]]) -> Rc<VecModel<Color>> {
    let colors: Vec<Color> = frame
        .iter()
        .map(|rgb| {
            if rgb[0] == 0 && rgb[1] == 0 && rgb[2] == 0 {
                Color::from_rgb_u8(25, 25, 35)
            } else {
                Color::from_rgb_u8(rgb[0], rgb[1], rgb[2])
            }
        })
        .collect();
    Rc::new(VecModel::from(colors))
}

fn main() {
    let effects = effects_common::capture_all(NUM_FRAMES);

    let window = MainWindow::new().unwrap();

    let current: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let fidx: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let effects = Rc::new(effects);

    // Initial frame
    window.set_led_colors(frame_to_colors(&effects[0].frames[0]).into());

    // Effect selector
    {
        let c = Rc::clone(&current);
        let f = Rc::clone(&fidx);
        window.on_select_effect(move |idx| {
            *c.borrow_mut() = idx as usize;
            *f.borrow_mut() = 0;
        });
    }

    // Animation timer
    let timer = slint::Timer::default();
    {
        let w = window.as_weak();
        let c = Rc::clone(&current);
        let f = Rc::clone(&fidx);
        let e = Rc::clone(&effects);

        timer.start(
            slint::TimerMode::Repeated,
            core::time::Duration::from_millis(80),
            move || {
                let Some(win) = w.upgrade() else { return };
                let ci = *c.borrow();
                let ef = &e[ci];
                let mut fi = f.borrow_mut();
                *fi = (*fi + 1) % ef.frames.len();

                win.set_led_colors(frame_to_colors(&ef.frames[*fi]).into());
                win.set_frame_info(SharedString::from(format!(
                    "{} · Frame {}/{} · {} LEDs",
                    ef.name,
                    *fi + 1,
                    ef.frames.len(),
                    ef.led_count
                )));
            },
        );
    }

    window.run().unwrap();
    drop(timer);
}
