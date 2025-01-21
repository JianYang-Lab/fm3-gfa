use anyhow::Result;
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    multi::many0,
    sequence::preceded,
    IResult,
};
use noodles::vcf::{
    self,
    variant::record::{
        info::field::value::{self, Array},
        Ids,
    },
    Header,
};
use regex::Regex;
use std::str::FromStr;

#[derive(Debug)]
pub struct BubbleVariant {
    pub id: String,
    pub pos: usize,
    pub allele_traversal: Vec<Traversal>,
}

#[derive(Debug, Clone)]
pub struct Traversal {
    pub nodes: Vec<u32>, // store node indices
}

pub fn parse_vcf_file(path: &str) -> Result<Vec<BubbleVariant>> {
    let mut variants = Vec::new();
    let mut reader = vcf::io::reader::Builder::default().build_from_path(path)?;
    let header = reader.read_header()?;

    for result in reader.records() {
        let record = result?;
        let variant = BubbleVariant::from_vcf_record(&record, &header)?;
        variants.push(variant);
    }

    Ok(variants)
}

impl BubbleVariant {
    // create BubbleVariant
    pub fn new(id: String, pos: usize, allele_traversal: Vec<Traversal>) -> Result<Self> {
        Ok(Self {
            id,
            pos,
            allele_traversal,
        })
    }

    // create BubbleVariant from VCF record
    pub fn from_vcf_record(record: &vcf::Record, header: &Header) -> Result<Self> {
        let id = Self::get_id(record)?;

        let pos = record.variant_start().transpose()?.unwrap().get();

        let allele_traversals = Self::get_allele_traversal(record, header)?;

        Self::new(id, pos, allele_traversals)
    }

    // aux function to get variant ID
    fn get_id(record: &vcf::Record) -> Result<String> {
        let id = match record.ids().iter().next() {
            Some(ids) => ids.to_string(),
            None => Err(anyhow::anyhow!("Variant ID is necessary"))?,
        };
        Ok(id)
    }

    // aux func to get allele traversal
    fn get_allele_traversal(record: &vcf::Record, header: &Header) -> Result<Vec<Traversal>> {
        let mut allele_traversals = Vec::new();
        match record.info().get(header, "AT") {
            Some(value) => {
                let value = value?;
                let array = match value {
                    Some(value::Value::Array(value)) => value,
                    _ => Err(anyhow::anyhow!(
                        "Allele traversal should be an array split by ','"
                    ))?,
                };
                let at_strs = match array {
                    Array::String(s) => s,
                    _ => Err(anyhow::anyhow!(
                        "Allele traversal should be an array of strings"
                    ))?,
                };
                for at_str in at_strs.iter() {
                    match at_str? {
                        Some(s) => {
                            let s = s.to_string();
                            let traversal =
                                Traversal::from_str(&s).map_err(|e| anyhow::anyhow!(e))?;
                            allele_traversals.push(traversal);
                        }
                        None => Err(anyhow::anyhow!(
                            "Allele traversal should be an array of strings"
                        ))?,
                    };
                }
            }

            None => Err(anyhow::anyhow!(
                "Allele traversal(AT) field in INFO is necessary"
            ))?,
        };

        Ok(allele_traversals)
    }
}

impl Traversal {
    // nom parser separated by > or <
    fn parse_separator(input: &str) -> IResult<&str, char> {
        alt((char('>'), char('<')))(input)
    }

    // nom parser for single node
    fn parse_single_node(input: &str) -> IResult<&str, u32> {
        let (input, num) = preceded(Self::parse_separator, digit1)(input)?;
        Ok((input, num.parse().unwrap()))
    }

    // nom parser for Traversal
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, first) = Self::parse_single_node(input)?;
        let (input, rest) = many0(Self::parse_single_node)(input)?;

        let mut nodes = vec![first];
        nodes.extend(rest);

        Ok((input, Traversal { nodes }))
    }

    // Heavy Regex parser, it's slower than nom, just for comparison
    fn _parse_regex(input: &str) -> Result<Self, String> {
        // build regex pattern
        let re = Regex::new(r"[>|<](\d+)").map_err(|e| e.to_string())?;

        let nodes: Vec<u32> = re
            .captures_iter(input)
            .filter_map(|cap| {
                cap.get(1) // capture group 1
                    .and_then(|m| m.as_str().parse().ok())
            })
            .collect();

        if nodes.is_empty() {
            Err("No valid nodes found in input".to_string())
        } else {
            Ok(Traversal { nodes })
        }
    }
}

impl FromStr for Traversal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s) {
            Ok(("", traversal)) => Ok(traversal),
            Ok((remainder, _)) => Err(format!("Unparsed input remaining: {}", remainder)),
            Err(e) => Err(format!("Parse error: {}", e)),
        }
        // Self::parse_regex(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_traversal() {
        // simple test
        let input = ">21610>21611>21612>21613>21614";
        let traversal = Traversal::from_str(input).unwrap();
        assert_eq!(traversal.nodes, vec![21610, 21611, 21612, 21613, 21614]);

        // `<` separator
        let input = ">2622646<2622651";
        let traversal = Traversal::from_str(input).unwrap();
        assert_eq!(traversal.nodes, vec![2622646, 2622651]);

        // single node
        let input = ">2622646";
        let traversal = Traversal::from_str(input).unwrap();
        assert_eq!(traversal.nodes, vec![2622646]);
    }
}
