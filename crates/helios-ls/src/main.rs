/// Prints the usage information of the Helios executable.
fn print_usage() {
    println!("{}", include_str!("../usage.txt"));
}

/// Prints the current version number of the Helios executable.
///
/// This function will print the version number found in the `Cargo.toml`
/// file of this package.
fn print_version() {
    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        println!("helios-ls {}", version);
    } else {
        eprintln!("ERROR: Failed to get version.");
    }
}

fn main() {
    // Initialize the logger
    env_logger::init();

    let mut args = std::env::args();
    args.next(); // Skip path to executable

    match (args.next(), args.next()) {
        (Some(arg), param) => match (&*arg, param) {
            ("-h", _) | ("--help", _) => print_usage(),
            ("-V", _) | ("--version", _) => print_version(),
            _ => {
                eprintln!("ERROR: Unrecognised option `{}`", arg);
                print_usage()
            }
        },
        _ => {
            log::trace!("Starting Helios language server...");
            helios_ls::start()
        }
    }
}