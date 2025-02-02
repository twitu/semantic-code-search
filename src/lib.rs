pub mod data;
use data::QueryOps;


pub struct Config {
    pub data_json: String,
    pub query_json: String,
    pub query: Vec<QueryOps>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        if args.len() < 3 {
            return Err("Too few arguments! Usage: <data_json_path> <queries_json_path>");
        }

        let data_json = args[1].clone();
        let query_json = args[2].clone();
        let query = match QueryOps::parse_query(&query_json) {
            Ok(q) => q,
            Err(e) => {
                println!("Could not parse query: {}", e);
                vec![]
            }
        };

        Ok(Config {
            data_json,
            query_json,
            query,
        })
    }
}
