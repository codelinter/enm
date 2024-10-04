mod vsetter;
mod node_pkg_j;
mod arch;
mod archive;
mod cli;
mod p_folder;
mod http;
mod lts;
mod path_ext;
mod version;
mod progress; 

#[macro_use]
mod llevel;
mod p_folders;

fn main() {
    env_logger::init();
    let value = crate::cli::parse();
    value.subcmd.call(value.config);
}
