use crate::Months;
use std::{fmt::Display, str::FromStr};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::EnumIter;

#[derive(Serialize, Clone, EnumIter, Deserialize, sqlx::Type)]
#[sqlx(type_name = "expense_type", rename_all = "lowercase")]
pub enum ExpenseType {
    Food,
    Transport,
    Health,
    Education,
    Entertainment,
    Other,
}

impl FromStr for ExpenseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Food" => Ok(ExpenseType::Food),
            "Transport" => Ok(ExpenseType::Transport),
            "Health" => Ok(ExpenseType::Health),
            "Education" => Ok(ExpenseType::Education),
            "Entertainment" => Ok(ExpenseType::Entertainment),
            "Other" => Ok(ExpenseType::Other),
            _ => Err(format!("{} is not a valid expense type", s)),
        }
    }
}

impl Display for ExpenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpenseType::Food => write!(f, "Food"),
            ExpenseType::Transport => write!(f, "Transport"),
            ExpenseType::Health => write!(f, "Health"),
            ExpenseType::Education => write!(f, "Education"),
            ExpenseType::Entertainment => write!(f, "Entertainment"),
            ExpenseType::Other => write!(f, "Other"),
        }
    }
}

#[derive(FromRow, Serialize)]
pub struct Expense {
    pub id: i32,
    pub description: String,
    pub price: f32,
    pub expense_type: ExpenseType,
    pub is_essencial: bool,
    pub date: NaiveDate,
}

#[derive(Deserialize, Debug)]
pub struct GetExpense {
    #[serde(deserialize_with = "empty_string_to_none")]
    pub month: Option<Months>,
}

fn empty_string_to_none<'de, D>(deserializer: D) -> Result<Option<Months>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Months::from_str(&s).unwrap()))
    }
}
