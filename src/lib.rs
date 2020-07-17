//! This crate has two parts. The first creates [`SharedBill`] s (which couple a
//! Bill with the amount that will be split evenly). The second takes [SharedBill]s
//! and creates invoices for each roommate.
//!
//! [`SharedBill`]: sharing/struct.SharedBill.html

mod bill;
mod interval;
mod invoice;
mod roommate;
mod shared_cost;
mod split;

pub use interval::{DateInterval, ResponsibilityInterval, ResponsibilityRecord};
pub use roommate::{Roommate, RoommateGroup};
pub mod sharing {
    pub use super::bill::Bill;
    pub use super::bill::SharedBill;
    pub use super::invoice::SharingData;
}
pub mod splitting {
    pub use super::invoice::Invoice;
    pub use super::split::ResponsibilitySplit;
}

use chrono::format::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoommatesError {
    #[error("The end of an interval cannot be before the start")]
    NegativeLengthInterval,

    #[error("Error parsing date")]
    InvalidDate {
        #[from]
        source: ParseError,
    },
}
