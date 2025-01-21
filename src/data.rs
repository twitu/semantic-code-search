use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub data_flows: Vec<DataFlow>,
    pub file_path: String,
    types: BTreeMap<String, Type>,
    type_vars: BTreeSet<String>,
}

impl Database {
    pub fn load_from_json(path: &str) -> Self {
        #[derive(Deserialize)]
        struct Wrapper {
            file_path: String,
            dataflow: Vec<Vec<UnitFlow>>,
        }

        let data = fs::read_to_string(path).expect("Could not read file");
        let parsed: Wrapper = serde_json::from_str(&data).expect("JSON parse error");

        let mut type_map: BTreeMap<String, Type> = BTreeMap::new();
        let mut type_vars = BTreeSet::new();

        for flow in &parsed.dataflow {
            for uf in flow {
                match uf {
                    UnitFlow::Type(t) => {
                        type_map.insert(t.name.clone(), t.clone());
                    }
                    UnitFlow::TypeVar(tv) => {
                        type_vars.insert(tv.name.clone());
                    }
                    _ => {}
                }
            }
        }

        Database {
            data_flows: parsed.dataflow,
            file_path: parsed.file_path,
            types: type_map,
            type_vars,
        }
    }

    pub fn match_unit_flow(&self, uf: &UnitFlow, query: &QueryOps) -> bool {
        match (uf, query) {
            (UnitFlow::TypeVar(tv), QueryOps::QTypeVar(count)) => {
                self.count_typevar_flows(&tv.name) == *count
            }
            (UnitFlow::Type(t), QueryOps::QType(q)) => t.name == q.name,
            (UnitFlow::ConstructorArg(c), QueryOps::QConstructorArg(q)) => c.name == q.name,
            (_, QueryOps::QDesc(d)) => match uf {
                UnitFlow::Type(t) => t.desc.as_deref() == Some(d),
                UnitFlow::ConstructorArg(c) => c.desc.as_deref() == Some(d),
                UnitFlow::TypeVar(tv) => tv.desc.as_deref() == Some(d),
                UnitFlow::ProgLoc(p) => p.desc.as_deref() == Some(d),
            },
            (_, QueryOps::Wildcard) => true,
            _ => false,
        }
    }

    pub fn match_flow(&self, flow: &[UnitFlow], query: &[QueryOps]) -> bool {
        match (flow, query) {
            // Wildcard matches all remaining flows
            (_, [QueryOps::Wildcard]) => true,
            (f, [QueryOps::Wildcard, next_query, rest @ ..]) => {
                // Try each position until we find a match for the next query item
                for (idx, unit_flow) in f.iter().enumerate() {
                    if self.match_unit_flow(unit_flow, next_query) {
                        // Found a match for the item after wildcard, try to match the rest
                        if self.match_flow(&f[idx + 1..], rest) {
                            return true;
                        }
                    }
                }

                false
            }
            ([fhead, frest @ ..], [qhead, qrest @ ..]) => {
                if self.match_unit_flow(fhead, qhead) {
                    self.match_flow(frest, qrest)
                } else {
                    false
                }
            }
            (_, []) => true,
            _ => false,
        }
    }

    pub fn count_typevar_flows(&self, typevar_name: &str) -> usize {
        self.data_flows
            .iter()
            .filter(|flow| {
                flow.iter().any(|unit| {
                    if let UnitFlow::TypeVar(tv) = unit {
                        tv.name == typevar_name
                    } else {
                        false
                    }
                })
            })
            .count()
    }
}
type DataFlow = Vec<UnitFlow>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Type {
    name: String,
    args: Vec<String>,
    /// Additional description about the specific flow
    desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstructorArg {
    name: String,
    arg_index: usize,
    desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgLoc {
    line: String,
    char_range: (usize, usize),
    desc: Option<String>,
}

impl ProgLoc {
    pub fn print_location(loc: &ProgLoc, itr: &usize) -> bool {
        if {
            loc.char_range.0 >= loc.line.len()
                || loc.char_range.1 > loc.line.len() + 1
                || loc.char_range.0 >= loc.char_range.1
        } {
            return false;
        }

        let line_text = &loc.line;
        let max_padding = 7;
        let mut itr_space = 0;
        if format!("{itr}").len() == 1 {
            itr_space = format!("[{}]  |", itr).len().min(max_padding);
        } else {
            itr_space = format!("[{}] |", itr).len().min(max_padding);
        }

        if format!("{itr}").len() == 1 {
            println!(
                "{} {}",
                format!("[{}]  |", itr).bright_blue(),
                line_text
            );
        } else {
            println!(
                "{} {}",
                format!("[{}] |", itr).bright_blue(),
                line_text
            );
        }

        let mut highlight = String::with_capacity(line_text.len());
        for i in 1..(line_text.len() + 1) {
            if i >= loc.char_range.0 && i < loc.char_range.1 {
                highlight.push('^');
            } else {
                highlight.push(' ');
            }
        }

        println!("{} {}", " ".repeat(itr_space), highlight.green());

        true
    }
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeVar {
    name: String,
    desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UnitFlow {
    Type(Type),
    ConstructorArg(ConstructorArg),
    TypeVar(TypeVar),
    ProgLoc(ProgLoc),
}

#[derive(serde::Deserialize, Debug)]
/// Match constructor argument in the data flow by name
pub struct QConstructorArg {
    name: String,
    /// Optionally match on specific argument unified
    arg_index: Option<usize>,
    /// Optionally match on description
    desc: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
/// Match type by name
pub struct QType {
    pub name: String,
    /// Optionally match on description
    pub desc: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub enum QueryOps {
    /// Match type variable by in-degree
    QTypeVar(usize),
    /// Match constructor argument in the data flow by name
    QConstructorArg(QConstructorArg),
    /// Match type by name
    QType(QType),
    /// Match based on string description for a [UnitFlow]
    QDesc(String),
    /// Match any unit flow
    Wildcard,
}
