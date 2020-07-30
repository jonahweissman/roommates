//! `roommates` is a library for splitting bills. It is designed to handle the
//! times when splitting bills evenly between all people in a housing unit
//! would be unfair, such as when not everyone is present for the whole billing
//! period.
//!
//! This crate has two parts. The first creates [`SharedBill`] s (which couple a
//! Bill with the amount that will be split evenly). The second takes [`SharedBill`]s
//! and creates invoices for each roommate.
//!
//! [`SharedBill`]: sharing/struct.SharedBill.html
//!
//! # Examples
//! ```
//! use roommates::{
//!     sharing::{Bill, SharedBill},
//!     splitting::CostSplit,
//!     ResponsibilityRecord,
//!     ResponsibilityInterval,
//!     DateInterval,
//!     RoommateGroup
//! };
//! use steel_cent::{Money, currency::USD};
//!
//! let house: RoommateGroup = vec!["Bob", "Joe"].into_iter().collect();
//! let water_bill = Bill::new_with_fixed_cost(
//!     Money::of_minor(USD, 100_00),
//!     DateInterval::from_strs("01/01/2020", "01/31/2020").unwrap(),
//!     Money::of_minor(USD, 20_00),
//! ).unwrap();
//! let water_bill = SharedBill::from_fixed(water_bill);
//! let responsibility: ResponsibilityRecord = vec![
//!     ResponsibilityInterval::new(
//!         house.borrow_by_name("Bob").unwrap(),
//!         DateInterval::from_strs("01/10/2020", "01/19/2020").unwrap(),
//!         0,
//!     ),
//!     ResponsibilityInterval::new(
//!         house.borrow_by_name("Joe").unwrap(),
//!         DateInterval::from_strs("01/10/2020", "01/14/2020").unwrap(),
//!         0,
//!     ),
//! ].into_iter().collect();
//! let water_bill_split = house.individual_responsibilities(
//!     &responsibility,
//!     water_bill.usage_period(),
//! );
//! let money_split = CostSplit::split_bill_list(vec![(&water_bill, &water_bill_split)]);
//! assert_eq!(
//!     money_split.get(house.borrow_by_name("Bob").unwrap()).unwrap(),
//!     Money::of_minor(USD, 63_33),
//! );
//! assert_eq!(
//!     money_split.get(house.borrow_by_name("Joe").unwrap()).unwrap(),
//!     Money::of_minor(USD, 36_66),
//! );
//! ```

mod bill;
mod interval;
mod roommate;
mod shared_cost;
mod split;

pub use interval::{DateInterval, IntervalError, ResponsibilityInterval, ResponsibilityRecord};
pub use roommate::{Roommate, RoommateGroup};

pub mod sharing {
    pub use super::bill::{Bill, BillError, SharedBill};
    pub use super::shared_cost::{
        convert_to_shared, convert_to_shared_ti, DependentVariable, EstimationError,
        IndependentVariable, Occupancy, OccupancyAndTemperature,
    };
}

pub mod splitting {
    pub use super::split::CostSplit;
    pub use super::split::ResponsibilitySplit;
}
