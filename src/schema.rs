use std::{
    fmt::{self, Display},
    str::FromStr,
};

use serde::{de, Deserialize, Serialize};
use strum::EnumIter;
use time::{macros::format_description, Date};

#[derive(Debug, PartialEq, Eq, Serialize, Clone, EnumIter, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "expense_category", rename_all = "lowercase")]
/// `ExpenseCategory` is an enum with the types of expenses.
pub enum ExpenseCategory {
    Restaurants,
    Shopping,
    Services,
    Entertainment,
    Groceries,
    Salary,
    #[serde(rename = "Interest Income")]
    InterestIncome,
    Utilities,
    Pharmacy,
    Transfer,
    Transport,
    #[default]
    Others,
    Investments,
}

impl FromStr for ExpenseCategory {
    type Err = String;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "Restaurants" => return Ok(Self::Restaurants),
            "Shopping" => return Ok(Self::Shopping),
            "Services" => return Ok(Self::Services),
            "Entertainment" => return Ok(Self::Entertainment),
            "Groceries" => return Ok(Self::Groceries),
            "Salary" => return Ok(Self::Salary),
            "Interest Income" => return Ok(Self::InterestIncome),
            "Utilities" => return Ok(Self::Utilities),
            "Pharmacy" => return Ok(Self::Pharmacy),
            "Transfer" => return Ok(Self::Transfer),
            "Transport" => return Ok(Self::Transport),
            "Others" => return Ok(Self::Others),
            "Investments" => return Ok(Self::Investments),
            _ => return Err(format!("{str} is not a valid expense type")),
        }
    }
}

impl Display for ExpenseCategory {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Restaurants => return write!(fmtr, "Restaurants"),
            Self::Shopping => return write!(fmtr, "Shopping"),
            Self::Services => return write!(fmtr, "Services"),
            Self::Entertainment => return write!(fmtr, "Entertainment"),
            Self::Groceries => return write!(fmtr, "Groceries"),
            Self::Salary => return write!(fmtr, "Salary"),
            Self::InterestIncome => return write!(fmtr, "Interest Income"),
            Self::Utilities => return write!(fmtr, "Utilities"),
            Self::Pharmacy => return write!(fmtr, "Pharmacy"),
            Self::Transfer => return write!(fmtr, "Transfer"),
            Self::Transport => return write!(fmtr, "Transport"),
            Self::Others => return write!(fmtr, "Others"),
            Self::Investments => return write!(fmtr, "Investments"),
        }
    }
}

#[derive(Deserialize, Debug)]
/// `GetExpense` is a struct with the fields of an expense that can be retrieved.
pub struct GetExpense {
    #[serde(deserialize_with = "empty_string_to_none")]
    pub from: Option<Date>,
    #[serde(deserialize_with = "empty_string_to_none")]
    pub to: Option<Date>,
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
    pub date: Option<Date>,
}

#[derive(Deserialize, Debug)]
pub struct CreateExpense {
    pub description: String,
    pub price: f32,
    pub category: ExpenseCategory,
    pub is_essential: Option<bool>,
    pub date: Date,
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
    pub date: Option<Date>,
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

/// Deserialization serde function that parses a Date from a string.
/// If the string is empty, returns None.
/// If the string is a valid date, returns Some(Date).
/// If the string is not a valid date, returns an error.
fn empty_string_to_none<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    if str.is_empty() {
        return Ok(None);
    }

    let format = format_description!("[year]-[month]-[day]");
    return Ok(Some(Date::parse(&str, &format).map_err(de::Error::custom)?));
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
