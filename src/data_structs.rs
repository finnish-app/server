use std::{fmt::Display, str::FromStr};

use chrono::Month;
use serde::Deserialize;
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Clone, Deserialize)]
pub enum Months {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl FromStr for Months {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "January" => Ok(Self::January),
            "February" => Ok(Self::February),
            "March" => Ok(Self::March),
            "April" => Ok(Self::April),
            "May" => Ok(Self::May),
            "June" => Ok(Self::June),
            "July" => Ok(Self::July),
            "August" => Ok(Self::August),
            "September" => Ok(Self::September),
            "October" => Ok(Self::October),
            "November" => Ok(Self::November),
            "December" => Ok(Self::December),
            _ => Err(format!("{s} is not a valid month")),
        }
    }
}

impl Display for Months {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::January => write!(f, "January"),
            Self::February => write!(f, "February"),
            Self::March => write!(f, "March"),
            Self::April => write!(f, "April"),
            Self::May => write!(f, "May"),
            Self::June => write!(f, "June"),
            Self::July => write!(f, "July"),
            Self::August => write!(f, "August"),
            Self::September => write!(f, "September"),
            Self::October => write!(f, "October"),
            Self::November => write!(f, "November"),
            Self::December => write!(f, "December"),
        }
    }
}

impl From<Months> for u32 {
    fn from(value: Months) -> Self {
        match value {
            Months::January => 1,
            Months::February => 2,
            Months::March => 3,
            Months::April => 4,
            Months::May => 5,
            Months::June => 6,
            Months::July => 7,
            Months::August => 8,
            Months::September => 9,
            Months::October => 10,
            Months::November => 11,
            Months::December => 12,
        }
    }
}

impl From<chrono::Month> for Months {
    fn from(value: chrono::Month) -> Self {
        match value {
            Month::January => Self::January,
            Month::February => Self::February,
            Month::March => Self::March,
            Month::April => Self::April,
            Month::May => Self::May,
            Month::June => Self::June,
            Month::July => Self::July,
            Month::August => Self::August,
            Month::September => Self::September,
            Month::October => Self::October,
            Month::November => Self::November,
            Month::December => Self::December,
        }
    }
}
