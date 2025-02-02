use colored::*;
use semantic_code_search::data::{Database, ProgLoc, QueryOps, UnitFlow};
use semantic_code_search::Config;

fn main() {
    let config = Config::build(&std::env::args().collect::<Vec<String>>())
        .expect("Failed to build configuration");

    let db = Database::load_from_json(&config.data_json);
    let query = config.query;
    let results = search_dataflows(&db, &query);
    println!("\n{}", "━".repeat(80).bright_black());
    if results.is_empty() {
        println!("{}", "No data flows matched the query.\n".bright_red());
    } else {
        println!(
            "{} {}\n",
            "Matched data flows:".bright_blue(),
            results.len()
        );
    }
    print_results(&results);
}

fn search_dataflows<'a>(db: &'a Database, query: &'a [QueryOps]) -> Vec<&'a Vec<UnitFlow>> {
    db.data_flows
        .iter()
        .filter(|flow| db.match_flow(flow, query))
        .collect()
}

fn print_results(results: &[&Vec<UnitFlow>]) {
    for (flow_idx, flow) in results.iter().enumerate() {
        let prog_locs: Vec<_> = flow
            .iter()
            .filter_map(|uf| match uf {
                UnitFlow::ProgLoc(pl) => Some(pl),
                _ => None,
            })
            .collect();

        if prog_locs.is_empty() {
            println!(
                "{}",
                "No program locations found for this data flow.".bright_red()
            );
            continue;
        }

        let mut itr = 1;
        for loc in prog_locs {
            if ProgLoc::print_location(loc, &itr) {
                itr += 1;
            };
        }

        if itr > 1 && flow_idx < results.len() - 1 {
            println!("{}", "━".repeat(80).bright_black());
        }
    }
}
