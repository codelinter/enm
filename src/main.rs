mod actions;
mod alias;
mod available_versions;
mod entry_interface;

#[macro_use]
mod std_system_structure;
mod ll_int;
mod version_std;

fn main() {
    env_logger::init();
    let value = crate::entry_interface::parse();
    value.valuator.call(value.app_cfg);
}
