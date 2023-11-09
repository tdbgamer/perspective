use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

// Copied from perspective-viewer until we can unify these workspaces.

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Scalar {
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(f64),
    Null,
    // // Can only have one u64 representation ...
    // Date(u64)
    // Int(u32)
}

impl Display for Scalar {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Float(x) => write!(fmt, "{}", x),
            Self::String(x) => write!(fmt, "{}", x),
            Self::Bool(x) => write!(fmt, "{}", x),
            Self::DateTime(x) => write!(fmt, "{}", x),
            Self::Null => write!(fmt, ""),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Deserialize, Debug, Eq, PartialEq, Serialize)]
#[serde()]
pub enum FilterOp {
    #[serde(rename = "contains")]
    Contains,

    #[serde(rename = "not in")]
    NotIn,

    #[serde(rename = "in")]
    In,

    #[serde(rename = "begins with")]
    BeginsWith,

    #[serde(rename = "ends with")]
    EndsWith,

    #[serde(rename = "is null")]
    IsNull,

    #[serde(rename = "is not null")]
    IsNotNull,

    #[serde(rename = ">")]
    GT,

    #[serde(rename = "<")]
    LT,

    #[serde(rename = "==")]
    EQ,

    #[serde(rename = ">=")]
    GTE,

    #[serde(rename = "<=")]
    LTE,

    #[serde(rename = "!=")]
    NE,
}

impl Display for FilterOp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let op = match self {
            Self::Contains => "contains",
            Self::In => "in",
            Self::NotIn => "not in",
            Self::BeginsWith => "begins with",
            Self::EndsWith => "ends with",
            Self::IsNull => "is null",
            Self::IsNotNull => "is not null",
            Self::GT => ">",
            Self::LT => "<",
            Self::EQ => "==",
            Self::GTE => ">=",
            Self::LTE => "<=",
            Self::NE => "!=",
        };

        write!(fmt, "{}", op)
    }
}

impl FromStr for FilterOp {
    type Err = String;

    fn from_str(input: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        match input {
            "contains" => Ok(Self::Contains),
            "in" => Ok(Self::In),
            "not in" => Ok(Self::NotIn),
            "begins with" => Ok(Self::BeginsWith),
            "ends with" => Ok(Self::EndsWith),
            "is null" => Ok(Self::IsNull),
            "is not null" => Ok(Self::IsNotNull),
            ">" => Ok(Self::GT),
            "<" => Ok(Self::LT),
            "==" => Ok(Self::EQ),
            ">=" => Ok(Self::GTE),
            "<=" => Ok(Self::LTE),
            "!=" => Ok(Self::NE),
            x => Err(format!("Unknown filter operator {}", x)),
        }
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FilterTerm {
    Scalar(Scalar),
    Array(Vec<Scalar>),
}

impl Display for FilterTerm {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Scalar(x) => {
                write!(fmt, "{}", x)?;
            }
            Self::Array(xs) => write!(
                fmt,
                "{}",
                Itertools::intersperse(xs.iter().map(|x| format!("{}", x)), ",".to_owned())
                    .collect::<String>()
            )?,
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde()]
pub struct Filter(pub String, pub FilterOp, pub FilterTerm);
#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
#[serde()]
pub struct Sort(pub String, pub SortDir);

#[derive(Clone, Copy, Deserialize, Debug, Eq, PartialEq, Serialize)]
#[serde()]
pub enum SortDir {
    #[serde(rename = "none")]
    None,

    #[serde(rename = "desc")]
    Desc,

    #[serde(rename = "asc")]
    Asc,

    #[serde(rename = "col desc")]
    ColDesc,

    #[serde(rename = "col asc")]
    ColAsc,

    #[serde(rename = "desc abs")]
    DescAbs,

    #[serde(rename = "asc abs")]
    AscAbs,

    #[serde(rename = "col desc abs")]
    ColDescAbs,

    #[serde(rename = "col asc abs")]
    ColAscAbs,
}

impl Display for SortDir {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                Self::None => "none",
                Self::Desc => "desc",
                Self::Asc => "asc",
                Self::ColDesc => "col desc",
                Self::ColAsc => "col asc",
                Self::DescAbs => "desc abs",
                Self::AscAbs => "asc abs",
                Self::ColDescAbs => "col desc abs",
                Self::ColAscAbs => "col asc abs",
            }
        )
    }
}

impl SortDir {
    /// Increment the `SortDir` in logical order, given an `abs()` modifier.
    pub fn cycle(&self, split_by: bool, abs: bool) -> Self {
        let order: &[Self] = match (split_by, abs) {
            (false, false) => &[Self::None, Self::Asc, Self::Desc],
            (false, true) => &[Self::None, Self::AscAbs, Self::DescAbs],
            (true, false) => &[
                Self::None,
                Self::Asc,
                Self::Desc,
                Self::ColAsc,
                Self::ColDesc,
            ],
            (true, true) => &[
                Self::None,
                Self::AscAbs,
                Self::DescAbs,
                Self::ColAscAbs,
                Self::ColDescAbs,
            ],
        };

        let index = order.iter().position(|x| x == self).unwrap_or(0);
        order[(index + 1) % order.len()]
    }
}

#[derive(Clone, Debug, Deserialize, Default, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewConfig {
    #[serde(default)]
    pub group_by: Vec<String>,

    #[serde(default)]
    pub split_by: Vec<String>,

    #[serde(default)]
    pub columns: Vec<Option<String>>,

    #[serde(default)]
    pub filter: Vec<Filter>,

    #[serde(default)]
    pub sort: Vec<Sort>,

    #[serde(default)]
    pub expressions: Vec<String>,

    #[serde(default)]
    pub aggregates: HashMap<String, Aggregate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde()]
pub enum SingleAggregate {
    #[serde(rename = "sum")]
    Sum,

    #[serde(rename = "sum abs")]
    SumAbs,

    #[serde(rename = "sum not null")]
    SumNotNull,

    #[serde(rename = "abs sum")]
    AbsSum,

    #[serde(rename = "pct sum parent")]
    PctSumParent,

    #[serde(rename = "pct sum grand total")]
    PctSumGrandTotal,

    #[serde(rename = "any")]
    Any,

    #[serde(rename = "unique")]
    Unique,

    #[serde(rename = "dominant")]
    Dominant,

    #[serde(rename = "median")]
    Median,

    #[serde(rename = "first")]
    First,

    #[serde(rename = "last by index")]
    LastByIndex,

    #[serde(rename = "last minus first")]
    LastMinusFirst,

    #[serde(rename = "last")]
    Last,

    #[serde(rename = "count")]
    Count,

    #[serde(rename = "distinct count")]
    DistinctCount,

    #[serde(rename = "avg")]
    Avg,

    #[serde(rename = "mean")]
    Mean,

    #[serde(rename = "join")]
    Join,

    #[serde(rename = "high")]
    High,

    #[serde(rename = "low")]
    Low,

    #[serde(rename = "max")]
    Max,

    #[serde(rename = "min")]
    Min,

    #[serde(rename = "high minus low")]
    HighMinusLow,

    #[serde(rename = "stddev")]
    StdDev,

    #[serde(rename = "var")]
    Var,
}

impl Display for SingleAggregate {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let term = match self {
            Self::Sum => "sum",
            Self::SumAbs => "sum abs",
            Self::SumNotNull => "sum not null",
            Self::AbsSum => "abs sum",
            Self::PctSumParent => "pct sum parent",
            Self::PctSumGrandTotal => "pct sum grand total",
            Self::Any => "any",
            Self::Unique => "unique",
            Self::Dominant => "dominant",
            Self::Median => "median",
            Self::First => "first",
            Self::LastByIndex => "last by index",
            Self::LastMinusFirst => "last minus first",
            Self::Last => "last",
            Self::Count => "count",
            Self::DistinctCount => "distinct count",
            Self::Avg => "avg",
            Self::Mean => "mean",
            Self::Join => "join",
            Self::High => "high",
            Self::Low => "low",
            Self::HighMinusLow => "high minus low",
            Self::StdDev => "stddev",
            Self::Var => "var",
            Self::Max => "max",
            Self::Min => "min",
        };

        write!(fmt, "{}", term)
    }
}

impl FromStr for SingleAggregate {
    type Err = ApiError;

    fn from_str(value: &str) -> ApiResult<Self> {
        match value {
            "sum" => Ok(Self::Sum),
            "sum abs" => Ok(Self::SumAbs),
            "sum not null" => Ok(Self::SumNotNull),
            "abs sum" => Ok(Self::AbsSum),
            "pct sum parent" => Ok(Self::PctSumParent),
            "pct sum grand total" => Ok(Self::PctSumGrandTotal),
            "any" => Ok(Self::Any),
            "unique" => Ok(Self::Unique),
            "dominant" => Ok(Self::Dominant),
            "median" => Ok(Self::Median),
            "first" => Ok(Self::First),
            "last by index" => Ok(Self::LastByIndex),
            "last minus first" => Ok(Self::LastMinusFirst),
            "last" => Ok(Self::Last),
            "count" => Ok(Self::Count),
            "distinct count" => Ok(Self::DistinctCount),
            "avg" => Ok(Self::Avg),
            "mean" => Ok(Self::Mean),
            "join" => Ok(Self::Join),
            "high" => Ok(Self::High),
            "low" => Ok(Self::Low),
            "max" => Ok(Self::Max),
            "min" => Ok(Self::Min),
            "high minus low" => Ok(Self::HighMinusLow),
            "stddev" => Ok(Self::StdDev),
            "var" => Ok(Self::Var),
            x => Err(format!("Unknown aggregate `{}`", x).into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde()]
pub enum MultiAggregate {
    #[serde(rename = "weighted mean")]
    WeightedMean,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum Aggregate {
    SingleAggregate(SingleAggregate),
    MultiAggregate(MultiAggregate, String),
}

impl Display for Aggregate {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::SingleAggregate(x) => write!(fmt, "{}", x)?,
            Self::MultiAggregate(MultiAggregate::WeightedMean, x) => {
                write!(fmt, "weighted mean by {}", x)?
            }
        };
        Ok(())
    }
}

impl FromStr for Aggregate {
    type Err = ApiError;

    fn from_str(input: &str) -> ApiResult<Self> {
        Ok(
            if let Some(stripped) = input.strip_prefix("weighted mean by ") {
                Self::MultiAggregate(MultiAggregate::WeightedMean, stripped.to_owned())
            } else {
                Self::SingleAggregate(SingleAggregate::from_str(input)?)
            },
        )
    }
}

const STRING_AGGREGATES: &[SingleAggregate] = &[
    SingleAggregate::Any,
    SingleAggregate::Count,
    SingleAggregate::DistinctCount,
    SingleAggregate::Dominant,
    SingleAggregate::First,
    SingleAggregate::Join,
    SingleAggregate::Last,
    SingleAggregate::LastByIndex,
    SingleAggregate::Median,
    SingleAggregate::Unique,
];

const NUMBER_AGGREGATES: &[SingleAggregate] = &[
    SingleAggregate::AbsSum,
    SingleAggregate::Any,
    SingleAggregate::Avg,
    SingleAggregate::Count,
    SingleAggregate::DistinctCount,
    SingleAggregate::Dominant,
    SingleAggregate::First,
    SingleAggregate::High,
    SingleAggregate::Low,
    SingleAggregate::Max,
    SingleAggregate::Min,
    SingleAggregate::HighMinusLow,
    SingleAggregate::LastByIndex,
    SingleAggregate::LastMinusFirst,
    SingleAggregate::Last,
    SingleAggregate::Mean,
    SingleAggregate::Median,
    SingleAggregate::PctSumParent,
    SingleAggregate::PctSumGrandTotal,
    SingleAggregate::StdDev,
    SingleAggregate::Sum,
    SingleAggregate::SumAbs,
    SingleAggregate::SumNotNull,
    SingleAggregate::Unique,
    SingleAggregate::Var,
];

impl Type {
    pub fn aggregates_iter(&self) -> Box<dyn Iterator<Item = Aggregate>> {
        match self {
            Self::Bool | Self::Date | Self::Datetime | Self::String => Box::new(
                STRING_AGGREGATES
                    .iter()
                    .map(|x| Aggregate::SingleAggregate(*x)),
            ),
            Self::Integer | Self::Float => Box::new(
                NUMBER_AGGREGATES
                    .iter()
                    .map(|x| Aggregate::SingleAggregate(*x)),
            ),
        }
    }

    pub const fn default_aggregate(&self) -> Aggregate {
        match self {
            Self::Bool | Self::Date | Self::Datetime | Self::String => {
                Aggregate::SingleAggregate(SingleAggregate::Count)
            }
            Self::Integer | Self::Float => Aggregate::SingleAggregate(SingleAggregate::Sum),
        }
    }
}
