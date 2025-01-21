//! This crate allows for reading [Graph Modeling Language (GML)](https://en.wikipedia.org/wiki/Graph_Modelling_Language) files.

use std::{error::Error, fmt::Display, str::FromStr};
extern crate pest;
use anyhow::Result;
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

#[derive(Debug)]
pub struct GMLError(String);

impl Error for GMLError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Display for GMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMLError: {}", self.0)
    }
}

#[derive(Parser, Debug)]
#[grammar = "grammar.pest"]
struct GMLParser;

#[derive(Debug, Clone, PartialEq)]
pub struct GMLObject {
    pub pairs: Vec<(String, GMLValue)>,
}
impl GMLObject {
    fn parse(obj: Pairs<'_, Rule>) -> Result<Self> {
        let mut current_key = None;
        let mut pairs = Vec::new();
        for entry in obj {
            match entry.as_rule() {
                Rule::creator_header => {
                    // ignore creator header
                    continue;
                }
                Rule::identifier => {
                    current_key = Some(entry.into_inner().as_str().to_owned());
                }
                Rule::value => {
                    let inner_value = entry
                        .into_inner()
                        .next()
                        .ok_or(GMLError("No rule inner value. Please report this.".into()))?;
                    match inner_value.as_rule() {
                        Rule::string => {
                            pairs.push((
                                current_key.clone().ok_or(GMLError(
                                    "String: No rule current key. Please report this.".into(),
                                ))?,
                                GMLValue::GMLString(inner_value.into_inner().as_str().to_string()),
                            ));
                        }
                        Rule::number => {
                            let num_str = inner_value.as_str();
                            // trying to parse as integer
                            if let Ok(n) = num_str.parse::<i64>() {
                                pairs.push((
                                    current_key.clone().ok_or(GMLError(
                                        "Number: No rule current key. Please report this".into(),
                                    ))?,
                                    GMLValue::GMLInt(n),
                                ));
                            } else {
                                // trying to parse as float
                                match num_str.parse::<f64>() {
                                    Ok(f) => {
                                        pairs.push((
                                            current_key.clone().ok_or(GMLError(
                                                "Number: No rule current key. Please report this"
                                                    .into(),
                                            ))?,
                                            GMLValue::GMLFloat(f),
                                        ));
                                    }
                                    Err(e) => {
                                        return anyhow::Result::Err(anyhow::anyhow!(
                                            "Failed to parse number: {:?}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                        Rule::object => {
                            pairs.push((
                                current_key.clone().ok_or(GMLError(
                                    "Object: No rule current key. Please report this".into(),
                                ))?,
                                GMLValue::GMLObject(Box::new(GMLObject::parse(
                                    inner_value.into_inner(),
                                )?)),
                            ));
                        }
                        _ => {
                            dbg!(inner_value.as_rule());
                            unreachable!()
                        }
                    }
                }
                Rule::EOI => {}
                _ => {
                    dbg!(entry.as_rule());
                    unreachable!()
                }
            }
        }
        Ok(GMLObject { pairs })
    }
}

impl FromStr for GMLObject {
    type Err = GMLError;

    fn from_str(text: &str) -> Result<GMLObject, GMLError> {
        let file = match GMLParser::parse(Rule::text, text) {
            Ok(k) => Ok(k),
            Err(e) => Err(GMLError(format!(
                "Failed to parse GML! (syntactic): {:?}",
                e
            ))),
        }?
        .next()
        .unwrap();
        match GMLObject::parse(file.into_inner()) {
            Ok(k) => Ok(k),
            Err(e) => Err(GMLError(format!(
                "Failed to parse GML! (semantic): {:?}",
                e
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMLValue {
    GMLString(String),
    GMLInt(i64),
    GMLFloat(f64),
    GMLObject(Box<GMLObject>),
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub directed: Option<bool>,
    pub id: Option<i64>,
    pub label: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub attrs: Vec<(String, GMLValue)>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: i64,
    pub label: Option<String>,
    pub attrs: Vec<(String, GMLValue)>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub source: i64,
    pub target: i64,
    pub label: Option<String>,
    pub attrs: Vec<(String, GMLValue)>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            directed: None,
            id: None,
            label: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            attrs: Vec::new(),
        }
    }

    // This turns the data into the object.
    // The other function is a wrapper to deal with the
    // outer graph[...] nonsense
    fn int_from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let id = int_take_attribute(&mut obj.pairs, "id");
        let id = if let Some(id) = id {
            let GMLValue::GMLInt(id) = id.1 else {
                return Err(GMLError(format!(
                    "Failed to parse graph id: {:?}. Expected int but found invalid type.",
                    id.1
                )));
            };
            Some(id)
        } else {
            None
        };
        let directed = int_take_attribute(&mut obj.pairs, "directed");
        let directed = if let Some(directed) = directed {
            let GMLValue::GMLInt(directed) = directed.1 else {
                return Err(GMLError(format!(
                    "Failed to parse graph directed: {:?}. Expected int but found invalid type.",
                    directed.1
                )));
            };
            Some(directed == 1)
        } else {
            None
        };

        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!(
                    "Failed to parse edge label: {:?}. Expected str but found invalid type.",
                    label.1
                )));
            };
            Some(label)
        } else {
            None
        };
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        while let Some((_, node)) = int_take_attribute(&mut obj.pairs, "node") {
            let GMLValue::GMLObject(node) = node else {
                return Err(GMLError(format!(
                    "Failed to parse node: {:?}. Expected object but found invalid type.",
                    node
                )));
            };
            nodes.push(Node::from_gml(*node)?);
        }
        while let Some((_, edge)) = int_take_attribute(&mut obj.pairs, "edge") {
            let GMLValue::GMLObject(edge) = edge else {
                return Err(GMLError(format!(
                    "Failed to parse edge: {:?}. Expected object but found invalid type.",
                    edge
                )));
            };
            edges.push(Edge::from_gml(*edge)?);
        }
        Ok(Graph {
            directed,
            id,
            label,
            nodes,
            edges,
            attrs: obj.pairs,
        })
    }
    /// Transform a [GMLObject] into a graph. This expects the root node
    /// of the graph.
    ///
    /// Note: This does not currently accept multiple graphs in a single file
    pub fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let graph = int_take_attribute(&mut obj.pairs, "graph");
        let Some(graph) = graph else {
            return Err(GMLError("Unable to parse graph from GMLObject".to_string()));
        };
        let GMLValue::GMLObject(graph) = graph.1 else {
            return Err(GMLError(format!(
                "Failed to parse graph: {:?}. Expected graph but found invalid type.",
                graph.1
            )));
        };
        Self::int_from_gml(*graph)
    }
}

impl Node {
    fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let id = int_take_attribute(&mut obj.pairs, "id");
        let Some(id) = id else {
            return Err(GMLError("Unable to parse id from node".to_string()));
        };
        let GMLValue::GMLInt(id) = id.1 else {
            return Err(GMLError(format!(
                "Failed to parse node id: {:?}. Expected int but found invalid type.",
                id.1
            )));
        };
        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!(
                    "Failed to parse edge label: {:?}. Expected str but found invalid type.",
                    label.1
                )));
            };
            Some(label)
        } else {
            None
        };
        Ok(Self {
            id,
            label,
            attrs: obj.pairs,
        })
    }

    pub fn get_graphic_pos(&self) -> (f64, f64) {
        // attrs: [("graphics", GMLObject(GMLObject { pairs: [("x", GMLFloat(157.0)), ("y", GMLFloat(137.0)), ("w", GMLFloat(0.0)), ("h", GMLFloat(0.0)), ("type", GMLString("rectangle")), ("width", GMLFloat(1.0))] }))]
        let graphics = match self.get_attribute("graphics") {
            Some((_, GMLValue::GMLObject(graphics))) => graphics,
            _ => return (0.0, 0.0),
        };

        let (x, y) =
            graphics
                .pairs
                .iter()
                .fold((0.0, 0.0), |(x, y), (k, v)| match (k.as_str(), v) {
                    ("x", GMLValue::GMLFloat(val)) => (*val, y),
                    ("y", GMLValue::GMLFloat(val)) => (x, *val),
                    _ => (x, y),
                });
        (x, y)
    }
}
impl Edge {
    fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let source = int_take_attribute(&mut obj.pairs, "source");
        let Some(source) = source else {
            return Err(GMLError("Unable to parse source from edge".to_string()));
        };
        let GMLValue::GMLInt(source) = source.1 else {
            return Err(GMLError(format!(
                "Failed to parse edge source id: {:?}. Expected int but found invalid type.",
                source.1
            )));
        };
        let target = int_take_attribute(&mut obj.pairs, "target");
        let Some(target) = target else {
            return Err(GMLError("Unable to parse target from edge".to_string()));
        };
        let GMLValue::GMLInt(target) = target.1 else {
            return Err(GMLError(format!(
                "Failed to parse edge source id: {:?}. Expected int but found invalid type.",
                target.1
            )));
        };
        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!(
                    "Failed to parse edge label: {:?}. Expected str but found invalid type.",
                    label.1
                )));
            };
            Some(label)
        } else {
            None
        };

        Ok(Self {
            source,
            target,
            label,
            attrs: obj.pairs,
        })
    }
}
pub trait HasGMLAttributes {
    fn attributes(&self) -> &Vec<(String, GMLValue)>;
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)>;
}

pub trait ReadableGMLAttributes<'a> {
    /// Take the attribute from the object if the key == name
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)>;
    /// Return a reference to the object if the key == name
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)>;
}
fn int_take_attribute(
    attrs: &mut Vec<(String, GMLValue)>,
    name: &str,
) -> Option<(String, GMLValue)> {
    let mut index = None;
    for (i, attr) in attrs.iter().enumerate() {
        if attr.0 == name {
            index = Some(i);
            break;
        }
    }
    index.map(|index| attrs.swap_remove(index))
}
fn int_get_attribute<'a>(
    attrs: &'a [(String, GMLValue)],
    name: &str,
) -> Option<&'a (String, GMLValue)> {
    attrs.iter().find(|&attr| attr.0 == name)
}
// Blanket impl is far better but it doesn't show up in the docs.
// impl<'a, T> ReadableGMLAttributes<'a> for T
// where
//     T: HasGMLAttributes,
// {
//     fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
//         let attrs = self.attributes_mut();
//         int_take_attribute(attrs, name)
//     }
//     fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
//         let attrs = self.attributes();
//         int_get_attribute(attrs, name)
//     }
// }
impl<'a> ReadableGMLAttributes<'a> for Node {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}
impl<'a> ReadableGMLAttributes<'a> for Edge {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}
impl<'a> ReadableGMLAttributes<'a> for Graph {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}

impl HasGMLAttributes for Node {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        &self.attrs
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        &mut self.attrs
    }
}
impl HasGMLAttributes for Edge {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        &self.attrs
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        &mut self.attrs
    }
}
impl HasGMLAttributes for Graph {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        &self.attrs
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        &mut self.attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::fs;
    #[test]
    fn parse_empty() {
        let file = "".to_string();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        assert!(Graph::from_gml(root).is_err());
    }
}
