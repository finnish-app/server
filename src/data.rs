use std::{fmt::Display, str::FromStr};

use chrono::Month;
use serde::Deserialize;
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Clone, Deserialize)]
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
            "January" => Ok(Months::January),
            "February" => Ok(Months::February),
            "March" => Ok(Months::March),
            "April" => Ok(Months::April),
            "May" => Ok(Months::May),
            "June" => Ok(Months::June),
            "July" => Ok(Months::July),
            "August" => Ok(Months::August),
            "September" => Ok(Months::September),
            "October" => Ok(Months::October),
            "November" => Ok(Months::November),
            "December" => Ok(Months::December),
            _ => Err(format!("{} is not a valid month", s)),
        }
    }
}

impl Display for Months {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Months::January => write!(f, "January"),
            Months::February => write!(f, "February"),
            Months::March => write!(f, "March"),
            Months::April => write!(f, "April"),
            Months::May => write!(f, "May"),
            Months::June => write!(f, "June"),
            Months::July => write!(f, "July"),
            Months::August => write!(f, "August"),
            Months::September => write!(f, "September"),
            Months::October => write!(f, "October"),
            Months::November => write!(f, "November"),
            Months::December => write!(f, "December"),
        }
    }
}

impl Months {
    pub fn from_chrono_month(month: Month) -> Self {
        match month {
            Month::January => Months::January,
            Month::February => Months::February,
            Month::March => Months::March,
            Month::April => Months::April,
            Month::May => Months::May,
            Month::June => Months::June,
            Month::July => Months::July,
            Month::August => Months::August,
            Month::September => Months::September,
            Month::October => Months::October,
            Month::November => Months::November,
            Month::December => Months::December,
        }
    }
}
