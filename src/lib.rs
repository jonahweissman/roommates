//! This crate has two parts. The first creates [`sharing::SharedBill`] s (which couple a
//! Bill with the amount that will be split evenly). The second takes [SharedBill]s
//! and creates invoices for each roommate.

mod bill;
mod interval;
mod invoice;
mod roommate;
mod shared_cost;
mod split;

pub use bill::Bill;
pub use roommate::{Roommate, RoommateGroup};
pub mod sharing {
    pub use super::bill::SharedBill;
    pub use super::interval::{DateInterval, ResponsibilityInterval, ResponsibilityRecord};
    pub use super::invoice::SharingData;
}
pub mod splitting {
    pub use super::invoice::Invoice;
    pub use super::split::ResponsibilitySplit;
}
