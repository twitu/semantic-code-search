pub mod data;
use data::QueryOps;
use serde_json;
use std::fs;

pub struct QueryReader;

impl QueryReader {
    pub fn read_from_file(path: &str) -> Vec<QueryOps> {
        let data = fs::read_to_string(path).expect("Failed to read queries file");
        serde_json::from_str(&data).expect("Failed to parse queries JSON")
    }
}
pub struct Config {
    pub data_json: String,
    pub queries_json: String,
    pub file_contents: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        if args.len() < 4 {
            return Err("Too few arguments! Usage: <data_json_path> <queries_json_path> <code_file_path>");
        }

        let data_json = args[1].clone();
        let queries_json = args[2].clone();
        let file_contents = args[3].clone();

        Ok(Config {
            data_json,
            queries_json,
            file_contents,
        })
    }
}
