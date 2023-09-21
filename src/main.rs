#![warn(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

mod vsetter;
mod node_pkg_j;
mod version_now;
mod version_installed;
mod version_uv_reader;
mod version_files;
mod enm_rnode_index;
mod content_fetcher;
mod arch;
mod cd_file_changer;
mod archive;
mod cli;
mod user_opted_version;
mod commands;
mod config;
mod p_folder;
mod fs;
mod http;
mod lts;
mod path_ext;
mod progress;
mod shell;
mod system_info;
mod system_version;
mod user_version;
mod version;

#[macro_use]
mod llevel;
mod default_version;
mod p_folders;

fn main() {
    env_logger::init();
    let value = crate::cli::parse();
    value.subcmd.call(value.config);
}
