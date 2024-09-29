use std::{
    fmt::{self, Display},
    str::FromStr,
};

use chrono::NaiveDate;
use serde::{de, Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Serialize, Clone, EnumIter, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "expense_category", rename_all = "lowercase")]
/// `ExpenseCategory` is an enum with the types of expenses.
pub enum ExpenseCategory {
    Food,
    Transport,
    Health,
    Education,
    Entertainment,
    #[default]
    Other,
}

impl FromStr for ExpenseCategory {
    type Err = String;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "Food" => return Ok(Self::Food),
            "Transport" => return Ok(Self::Transport),
            "Health" => return Ok(Self::Health),
            "Education" => return Ok(Self::Education),
            "Entertainment" => return Ok(Self::Entertainment),
            "Other" => return Ok(Self::Other),
            _ => return Err(format!("{str} is not a valid expense type")),
        }
    }
}

impl Display for ExpenseCategory {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Food => return write!(fmtr, "Food"),
            Self::Transport => return write!(fmtr, "Transport"),
            Self::Health => return write!(fmtr, "Health"),
            Self::Education => return write!(fmtr, "Education"),
            Self::Entertainment => return write!(fmtr, "Entertainment"),
            Self::Other => return write!(fmtr, "Other"),
        }
    }
}

#[derive(Deserialize, Debug)]
/// `GetExpense` is a struct with the fields of an expense that can be retrieved.
pub struct GetExpense {
    #[serde(deserialize_with = "empty_string_to_none")]
    pub from: Option<NaiveDate>,
    #[serde(deserialize_with = "empty_string_to_none")]
    pub to: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
/// `UpdateExpense` is a struct with the fields of an expense that can be updated.
/// All fields are optional.
/// This is part of the contract of the Data API.
pub struct UpdateExpenseApi {
    pub description: Option<String>,
    pub price: Option<f32>,
    pub category: Option<ExpenseCategory>,
    pub is_essential: Option<bool>,
    pub date: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct CreateExpense {
    pub description: String,
    pub price: f32,
    pub category: ExpenseCategory,
    pub is_essential: bool,
    pub date: NaiveDate,
}

#[derive(Deserialize, Debug)]
/// `UpdateExpense` is a struct with the fields of an expense that can be updated.
/// All fields are optional.
/// This is part of the contract of the Hypermedia API.
pub struct UpdateExpenseHypr {
    pub description: Option<String>,
    #[serde(deserialize_with = "de_string_to_option_f32")]
    pub price: Option<f32>,
    pub category: Option<ExpenseCategory>,
    #[serde(
        default = "default_is_essential_to_false",
        deserialize_with = "de_string_to_bool"
    )]
    pub is_essential: Option<bool>,
    pub date: Option<NaiveDate>,
}

/// Function to set the default value for `is_essential` in `UpdateExpense` to be Some(false).
#[expect(
    clippy::unnecessary_wraps,
    reason = "Needs to return option for custom deserializer"
)]
const fn default_is_essential_to_false() -> Option<bool> {
    return Some(false);
}

/// Deserialization serde function that parses a bool from a string.
/// If the string is empty, returns None.
/// If the string is 'true', returns Some(true).
/// If the string is 'false', returns Some(false).
/// If the string is not 'true' or 'false', returns an error.
fn de_string_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    if str.is_empty() {
        return Ok(None);
    }
    return Ok(Some(str.parse::<bool>().map_err(de::Error::custom)?));
}

/// Deserialization serde function that parses a Months from a string.
/// If the string is empty, returns None.
/// If the string is a valid month, returns Some(Months).
/// If the string is not a valid month, returns an error.
fn empty_string_to_none<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    if str.is_empty() {
        return Ok(None);
    }
    return Ok(Some(
        NaiveDate::parse_from_str(&str, "%Y-%m-%d").map_err(de::Error::custom)?,
    ));
}

/// Deserialization serde function that parses a f32 from a string.
/// If the string is empty, returns None.
/// If the string is a valid f32, returns Some(f32).
/// If the string is not a valid f32, returns an error.
fn de_string_to_option_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    if str.is_empty() {
        return Ok(None);
    }
    return Ok(Some(str.parse::<f32>().map_err(de::Error::custom)?));
}
