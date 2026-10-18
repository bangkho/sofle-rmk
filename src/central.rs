#![no_main]
#![no_std]

use rmk::macros::rmk_central;

// The central owns the only OLED fitted on this keyboard.
mod render;

#[rmk_central]
mod keyboard_central {}
