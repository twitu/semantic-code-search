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
            ([], []) => true,
            (_, []) => false,
            ([], _) => false,
            (f, q @ [QueryOps::Wildcard, rest @ ..]) => {
                let wildcard_count = q
                    .iter()
                    .take_while(|&op| matches!(op, QueryOps::Wildcard))
                    .count();

                let remaining_query = &q[wildcard_count..];

                for skip_count in 0..=f.len() {
                    if self.match_flow(&f[skip_count..], remaining_query) {
                        return true;
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
    line: usize,
    char_range: (usize, usize),
    desc: Option<String>,
}

impl ProgLoc {
    pub fn print_location(loc: &ProgLoc, lines: &[&str], itr: &usize) -> bool {
        if loc.line == 0
            || loc.line > lines.len()
            || loc.char_range.0 >= lines[loc.line - 1].len()
            || loc.char_range.1 > lines[loc.line - 1].len()
            || loc.char_range.0 >= loc.char_range.1
        {
            // println!("Invalid location: {:?}", loc);
            return false;
        }

        let line_text = lines[loc.line - 1];

        println!(
            "{} {}",
            format!("[{}]", itr).bright_blue(),
            format!("l.{}:{},{}", loc.line, loc.char_range.0, loc.char_range.1).yellow(),
        );

        let padding = 4;
        let line_num = format!("{:>padding$}", loc.line);
        println!(
            "{} {}  {}",
            line_num.bright_black(),
            "│".bright_black(),
            line_text
        );

        let mut highlight = String::with_capacity(line_text.len());
        for i in 0..line_text.len() {
            if i >= loc.char_range.0 && i < loc.char_range.1 {
                highlight.push('^');
            } else {
                highlight.push(' ');
            }
        }

        println!(
            "{} {}  {}",
            " ".repeat(padding),
            "│".bright_black(),
            highlight.green()
        );

        println!();
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

#[derive(serde::Deserialize)]
/// Match constructor argument in the data flow by name
pub struct QConstructorArg {
    name: String,
    /// Optionally match on specific argument unified
    arg_index: Option<usize>,
    /// Optionally match on description
    desc: Option<String>,
}

#[derive(serde::Deserialize)]
/// Match type by name
pub struct QType {
    pub name: String,
    /// Optionally match on description
    pub desc: Option<String>,
}

#[derive(serde::Deserialize)]
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
