use semantic_code_search::data::{Database, ProgLoc, QueryOps, UnitFlow};
use semantic_code_search::{Config, QueryReader};
use std::fs;

fn main() {
    let config = Config::build(&std::env::args().collect::<Vec<String>>())
        .expect("Failed to build configuration");

    let db = Database::load_from_json(&config.data_json);
    let query = QueryReader::read_from_file(&config.queries_json);
    let file_contents =
        fs::read_to_string(&config.file_contents).expect("Failed to read file contents");
    let lines: Vec<&str> = file_contents.lines().collect();

    let results = search_dataflows(&db, &query);
    println!("Matched data flows:");
    print_results(&results, &lines);
}

fn search_dataflows<'a>(db: &'a Database, query: &'a [QueryOps]) -> Vec<&'a Vec<UnitFlow>> {
    db.data_flows
        .iter()
        .filter(|flow| Database::match_flow(flow, query))
        .collect()
}

fn print_results(results: &[&Vec<UnitFlow>], lines: &[&str]) {
    for flow in results {
        let prog_locs = flow.iter().filter_map(|uf| match uf {
            UnitFlow::ProgLoc(pl) => Some(pl),
            _ => None,
        });

        for loc in prog_locs {
            ProgLoc::print_location(loc, lines);
        }
    }
}
