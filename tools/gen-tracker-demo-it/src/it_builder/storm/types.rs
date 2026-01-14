//! Type definitions for Nether Storm generator

#[derive(Clone, Copy, PartialEq)]
pub enum Section {
    DropA,
    DropB,
}

#[derive(Clone, Copy)]
pub enum BreakStyle {
    None,
    Ghost,
    Accent,
    Fill,
}
