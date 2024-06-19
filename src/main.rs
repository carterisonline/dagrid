#![feature(macro_metavar_expr)]

#[cfg(feature = "standalone")]
use nih_plug::nih_export_standalone;

#[cfg(feature = "standalone")]
fn main() {
    #[path = "./lib.rs"]
    mod dagrid;

    nih_export_standalone::<dagrid::DaGrid>();
}

#[cfg(not(feature = "standalone"))]
fn main() {
    eprintln!("Run this with the standalone feature to actually work")
}
