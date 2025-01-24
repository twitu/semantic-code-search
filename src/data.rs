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
            (UnitFlow::ConstructorArg(c), QueryOps::QConstructorArg(q)) => {
                c.name == q.name && q.arg_index.map_or(true, |idx| c.arg_index == idx)
            }
            (_, QueryOps::QDesc(d)) => match uf {
                UnitFlow::Type(t) => t.desc.as_deref() == Some(d),
                UnitFlow::ConstructorArg(c) => c.desc.as_deref() == Some(d),
                UnitFlow::TypeVar(tv) => tv.desc.as_deref() == Some(d),
                UnitFlow::ProgLoc(p) => p.desc.as_deref() == Some(d),
            },
            _ => false,
        }
    }

    pub fn match_flow(&self, flow: &[UnitFlow], query: &[QueryOps]) -> bool {
        match (flow, query) {
            (f, [next_query, rest @ ..]) => {
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
                "{}{} {}",
                format!("[{}]  ", itr).bright_blue(),
                format!("│").bright_black(),
                line_text
            );
        } else {
            println!(
                "{}{} {}",
                format!("[{}] ", itr).bright_blue(),
                format!("│").bright_black(),
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

        println!(
            "{}{} {}",
            " ".repeat(itr_space - 1),
            "└".bright_black(),
            highlight.green()
        );

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

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
/// Match constructor argument in the data flow by name
pub struct QConstructorArg {
    pub name: String,
    /// Optionally match on specific argument unified
    pub arg_index: Option<usize>,
    /// Optionally match on description
    pub desc: Option<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
/// Match type by name
pub struct QType {
    pub name: String,
    /// Optionally match on description
    pub desc: Option<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub enum QueryOps {
    /// Match type variable by in-degree
    QTypeVar(usize),
    /// Match constructor argument in the data flow by name
    QConstructorArg(QConstructorArg),
    /// Match type by name
    QType(QType),
    /// Match based on string description for a [UnitFlow]
    QDesc(String),
}

/// A simplified parser for query language
/// Examples:
///   #2          -> QTypeVar(2) (# for count/number)
///   List        -> QType(List)
///   List:desc   -> QType(List) with description
///   @x          -> QConstructorArg(x)
///   @x.1        -> QConstructorArg(x) at index 1
///   @x:desc     -> QConstructorArg(x) with description
///   "desc"      -> QDesc(desc)
impl QueryOps {
    fn parse_token(token: &str) -> Result<QueryOps, String> {
        match token.trim() {
            // Handle type variable count: #2
            s if s.starts_with('#') => s[1..]
                .parse()
                .map(QueryOps::QTypeVar)
                .map_err(|_| "Invalid type variable count".to_string()),

            // Handle constructor arg: @x, @x.1, @x:desc
            s if s.starts_with('@') => {
                let parts: Vec<&str> = s[1..].split(|c| c == '.' || c == ':').collect();
                match parts.as_slice() {
                    [name] => Ok(QueryOps::QConstructorArg(QConstructorArg {
                        name: name.to_string(),
                        arg_index: None,
                        desc: None,
                    })),
                    [name, idx] if idx.parse::<usize>().is_ok() => {
                        Ok(QueryOps::QConstructorArg(QConstructorArg {
                            name: name.to_string(),
                            arg_index: Some(idx.parse().unwrap()),
                            desc: None,
                        }))
                    }
                    [name, desc] => Ok(QueryOps::QConstructorArg(QConstructorArg {
                        name: name.to_string(),
                        arg_index: None,
                        desc: Some(desc.to_string()),
                    })),
                    _ => Err("Invalid constructor arg syntax".to_string()),
                }
            }

            // Handle quoted description: "desc"
            s if s.starts_with('"') && s.ends_with('"') => {
                Ok(QueryOps::QDesc(s[1..s.len() - 1].to_string()))
            }

            // Handle type: List or List:desc
            s => {
                let parts: Vec<&str> = s.split(':').collect();
                match parts.as_slice() {
                    [name] => Ok(QueryOps::QType(QType {
                        name: name.to_string(),
                        desc: None,
                    })),
                    [name, desc] => Ok(QueryOps::QType(QType {
                        name: name.to_string(),
                        desc: Some(desc.to_string()),
                    })),
                    _ => Err("Invalid type syntax".to_string()),
                }
            }
        }
    }

    pub fn parse_query(input: &str) -> Result<Vec<QueryOps>, String> {
        input
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Self::parse_token)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_query_parsing() {
        assert!(matches!(
            QueryOps::parse_query("#3").unwrap()[0],
            QueryOps::QTypeVar(3)
        ));

        // Test type patterns
        let query = QueryOps::parse_query("List:generic").unwrap();
        if let QueryOps::QType(qt) = &query[0] {
            assert_eq!(qt.name, "List");
            assert_eq!(qt.desc.as_deref(), Some("generic"));
        }

        // Test constructor arg patterns
        let query = QueryOps::parse_query("@x.1").unwrap();
        if let QueryOps::QConstructorArg(qa) = &query[0] {
            assert_eq!(qa.name, "x");
            assert_eq!(qa.arg_index, Some(1));
        }

        // Test description
        let query = QueryOps::parse_query("\"some desc\"").unwrap();
        if let QueryOps::QDesc(desc) = &query[0] {
            assert_eq!(desc, "some desc");
        }

        // Test complex query
        let query = QueryOps::parse_query("List, @x.2, \"foo bar\"").unwrap();
        assert_eq!(
            query,
            vec![
                QueryOps::QType(QType {
                    name: "List".to_string(),
                    desc: None,
                }),
                QueryOps::QConstructorArg(QConstructorArg {
                    name: "x".to_string(),
                    arg_index: Some(2),
                    desc: None
                }),
                QueryOps::QDesc("foo bar".to_string())
            ]
        );

        let query = QueryOps::parse_query("bool,\"if-then-else condition\"").unwrap();
        assert_eq!(
            query,
            vec![
                QueryOps::QType(QType {
                    name: "bool".to_string(),
                    desc: None,
                }),
                QueryOps::QDesc("if-then-else condition".to_string())
            ]
        );

        let query = QueryOps::parse_query("@Tuple.2").unwrap();
        assert_eq!(
            query,
            vec![QueryOps::QConstructorArg(QConstructorArg {
                name: "Tuple".to_string(),
                arg_index: Some(2),
                desc: None,
            })]
        );

        let query = QueryOps::parse_query("bool,@Tuple.1,\"if-then-else condition\"").unwrap();
        assert_eq!(
            query,
            vec![
                QueryOps::QType(QType {
                    name: "bool".to_string(),
                    desc: None,
                }),
                QueryOps::QConstructorArg(QConstructorArg {
                    name: "Tuple".to_string(),
                    arg_index: Some(1),
                    desc: None,
                }),
                QueryOps::QDesc("if-then-else condition".to_string())
            ]
        );
    }
}
