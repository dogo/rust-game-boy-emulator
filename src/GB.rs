#![allow(non_snake_case)]

pub mod RAM;
pub mod CPU;
pub mod instructions;
pub mod registers;
pub mod trace;

#[cfg(test)]
mod joypad_test;