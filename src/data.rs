/// Data types and Query operators
///
/// This module describes the core type system information being described
/// in data flows. It also defined the query operators that can match
/// individual unit flows. Here are some examples.
///
/// Example 1: Find where a boolean is used as the condition of an if statement
///
/// let even x = x `mod` 2 == 0
/// ...
/// if even 3 then print "even" else print "odd"
///
/// psuedo data flow:
/// Type("bool"), prog locations..., TypeVar("a1"), ConstructorArg("Function", 1, desc="even"), Type("Function"), ..., Type("Function"), ConstructorArg("Function", 1, desc="even"), ProgLoc(..., desc="if-then-else"), Type("bool")
/// search query:
/// QType("bool"), wildcard, QDesc("if-then-else")
///
///
/// Example 2: Find where the boolean in the right argument of a tuple is used as the condition of if statement
///
/// let version: string -> (int, int)
/// let breaking_change (maj, min) = (maj > 2, min == 17)
/// let deploy package =
///     let (_, minor_version_breaking) = breaking_change (version package)
///     if minor_version_breaking
///         then print("don't deploy")
///         else print("deploy")
///
/// psuedo data flow:
/// Type("bool"), prog locations....TypeVar("a") ConstructorArg("Tuple", 1), Type("Tuple")...Type("Tuple"), ConstructorArg("Tuple", 1), TypeVar("b").. prog locations.. ProgLoc(... desc="if-then-else"), Type("bool")
/// search query:
/// QType("bool"), wildcard, ConstructorArg("Tuple", 1), wildcard, ProgLoc(... desc="if-then-else")
///
/// Example 3: Find all functions whose name starts with version that take a tuple with left argument bool as input
///
/// let version_check (is_stable, count) =
///     if is_stable then count + 1 else count
///
/// let version_validate (is_released, name) =
///     if is_released
///     then print_string ("Released: " ^ name)
///     else print_string ("Unreleased: " ^ name)
///
/// psuedo data flow:
/// Type("bool"), ConstructorArg("Tuple", 0), Type("Tuple"), ConstructorArg("Function", 1, desc="version_check"), Type("Function")
/// search query:
/// QType("bool"), *, ConstructorArg("Tuple", 0), *, QType("Tuple"), *, QConstructorArg("Function", desc="version*")
///
/// Example 4: Find all functions whose output is a tuple returned by another function called correctness_check,
/// where the output type variable has in-degree 1
///
/// let correctness_check file =
///     (file.syntax_valid, file.line_count)
///
/// let performance_check file =
///     (file.is_optimized, file.exec_time)
///
/// let analyze_module module =
///     let stats = correctness_check module in  (* stats is used only once *)
///     stats
///
/// let analyze_module_complex module =
///     let stats1 = correctness_check module in
///     let stats2 = performance_check module in
///     if is_priority module
///     then stats1    (* output could be from either correctness_check *)
///     else stats2    (* or performance_check, making in-degree 2 *)
///
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    data_flows: Vec<DataFlow>,
    types: BTreeMap<String, Type>,
    type_vars: BTreeSet<String>,
}

impl Database {
    pub fn load_from_json(path: &str) -> Self {
        #[derive(Deserialize)]
        struct Wrapper {
            dataflow: Vec<UnitFlow>,
        }

        let data = fs::read_to_string(path).expect("Could not read file");
        let parsed: Wrapper = serde_json::from_str(&data).expect("JSON parse error");

        let mut type_map: BTreeMap<String, Type> = BTreeMap::new();
        let mut type_vars = BTreeSet::new();

        for uf in &parsed.dataflow {
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

        Database {
            data_flows: vec![parsed.dataflow],
            types: type_map,
            type_vars,
        }
    }

    pub fn match_unit_flow(uf: &UnitFlow, query: &QueryOps) -> bool {
        match (uf, query) {
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

    pub fn match_flow(flow: &[UnitFlow], query: &[QueryOps]) -> bool {
        match (flow, query) {
            ([], []) => true,
            (_, []) => false,
            ([], _) => false,
            (f, [QueryOps::Wildcard, rest @ ..]) => {
                for skip_count in 1..=f.len() {
                    if Self::match_flow(&f[skip_count..], rest) {
                        return true;
                    }
                }
                false
            }
            ([fhead, frest @ ..], [qhead, qrest @ ..]) => {
                if Self::match_unit_flow(fhead, qhead) {
                    Self::match_flow(frest, qrest)
                } else {
                    false
                }
            }
        }
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

/// Match constructor argument in the data flow by name
pub struct QConstructorArg {
    name: String,
    /// Optionally match on specific argument unified
    arg_index: Option<usize>,
    /// Optionally match on description
    desc: Option<String>,
}

/// Match type by name
pub struct QType {
    name: String,
    /// Optionally match on description
    desc: Option<String>,
}

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
