use std::fmt::Display;

use chrono::Month;
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Clone)]
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
