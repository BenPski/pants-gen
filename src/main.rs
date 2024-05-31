use std::process::exit;

use pants_gen::cli::CliArgs;

fn main() {
    let password = CliArgs::run();
    if let Some(p) = password {
        println!("{}", p);
    } else {
        println!("Contraints couldn't be met try again");
        exit(1)
    }
}
