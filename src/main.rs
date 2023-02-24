#![warn(clippy::pedantic, rust_2018_idioms, clippy::all)]
#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

mod entry_interface;

#[macro_use]
mod ll_int;

fn main() {
    env_logger::init();
    let value = crate::entry_interface::parse();
    value.valuator.call(value.app_cfg);
}
