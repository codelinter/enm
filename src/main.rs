#![warn(clippy::pedantic, rust_2018_idioms, clippy::all)]
#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

mod actions;
mod alias;
mod available_versions;
mod entry_interface;
mod app_config;
mod cpu_arch;
mod fetcher;
mod symlinked;
mod http;
mod loaders;
mod long_term_usage;

#[macro_use]
mod std_system_structure;
mod ll_int;
mod version_std;

fn main() {
    env_logger::init();
    let value = crate::entry_interface::parse();
    value.valuator.call(value.app_cfg);
}
