use std::{env, process};
use semantic_code_search::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        process::exit(1);
    });
    if let Err(e) = semantic_code_search::run(config){
        eprintln!("An error occured : {e}");
        process::exit(1);
    }
}
