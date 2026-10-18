//! Status screen for the one half that has an OLED fitted (the central).
//!
//! RMK's built-in `OledRenderer` folds the split and host links into a single
//! indicator (`central_connected || BLE connected`), which on the central shows
//! the *host* link and hides whether the peripheral half is up — the state you
//! actually watch while pairing. This renderer prints this half's battery, the
//! peripheral's battery (relayed over the split link), and the split and host
//! link states side by side.
//!
//! Labels are role-based (`bat` / `peer`) so the same code stays truthful if a
//! panel is ever fitted to the other half.

use core::fmt::Write;

use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use rmk::{
    display::{DisplayRenderer, RenderContext},
    heapless::String,
    types::{
        battery::{BatteryStatus, ChargeState},
        ble::BleState,
    },
};

/// Same glyphs as the built-in renderer: 6px advance, 8px tall.
const STYLE: MonoTextStyle<'_, BinaryColor> = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
/// Left margin, and the baselines of the three rows inside a 128x32 panel.
const X: i32 = 1;
const BASELINE: [i32; 3] = [9, 18, 27];

/// This half's battery, the peer's battery, then both link states.
#[derive(Default)]
pub struct SplitStatusRenderer;

impl DisplayRenderer<BinaryColor> for SplitStatusRenderer {
    fn render<D: DrawTarget<Color = BinaryColor>>(&mut self, ctx: &RenderContext, display: &mut D) {
        display.clear(BinaryColor::Off).ok();

        // A dark panel asleep is cheaper than a dark panel redrawn 10x/s.
        if ctx.sleeping {
            return;
        }

        let mut line: String<24> = String::new();

        // Row 1 — this half.
        write!(line, "bat {}", level_text(*ctx.battery)).ok();
        Text::new(&line, Point::new(X, BASELINE[0]), STYLE)
            .draw(display)
            .ok();

        // Row 2 — the peer. Its level only ever arrives over the split link, so
        // this reads `--` until the peripheral has reported once.
        line.clear();
        write!(line, "peer {}", level_text(*ctx.peripheral_batteries[0])).ok();
        Text::new(&line, Point::new(X, BASELINE[1]), STYLE)
            .draw(display)
            .ok();

        // Row 3 — split link, then which host the central is talking to.
        line.clear();
        match ctx.ble_status.state {
            BleState::Connected => write!(line, "{}  H{}", split_text(ctx), ctx.ble_status.profile).ok(),
            BleState::Advertising => write!(line, "{}  ADV", split_text(ctx)).ok(),
            BleState::Inactive => write!(line, "{}  USB", split_text(ctx)).ok(),
        };
        Text::new(&line, Point::new(X, BASELINE[2]), STYLE)
            .draw(display)
            .ok();
    }
}

/// Whether any peripheral half is paired to this central.
fn split_text(ctx: &RenderContext) -> &'static str {
    if ctx.peripherals_connected.iter().any(|&up| up) {
        "SPLIT OK"
    } else {
        "SPLIT LOST"
    }
}

/// `85%`, `+85%` while charging, or `--` when there is nothing to report.
fn level_text(status: BatteryStatus) -> String<8> {
    let mut text: String<8> = String::new();
    match status {
        BatteryStatus::Available {
            charge_state: ChargeState::Charging,
            level: Some(level),
        } => {
            write!(text, "+{}%", level).ok();
        }
        BatteryStatus::Available {
            level: Some(level), ..
        } => {
            write!(text, "{}%", level).ok();
        }
        BatteryStatus::Available { level: None, .. } | BatteryStatus::Unavailable => {
            text.push_str("--").ok();
        }
    };
    text
}
