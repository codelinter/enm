mod cli;

fn main() {
    env_logger::init();
    let value = crate::cli::parse();
    value.subcmd.call(value.config);
}
