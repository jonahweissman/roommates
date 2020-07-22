use steel_cent::Money;

use super::interval::DateInterval;
use crate::{Error, InvalidFixedCost::*};

/// Stores information about a bill
#[derive(Debug, Clone)]
pub struct Bill {
    amount_due: Money,
    fixed_cost: Money,
    usage_period: DateInterval,
}

impl Bill {
    /// Creates a new `Bill`
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::Bill};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// ```
    pub fn new(amount_due: Money, usage_period: DateInterval) -> Self {
        Bill::new_with_fixed_cost(amount_due, usage_period, Money::zero(amount_due.currency))
            .unwrap()
    }

    /// Creates a new `Bill` with a fixed cost
    ///
    /// Fixed cost must be in the same currency as the amount due and
    /// cannot be negative or greater than the amount due.
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::Bill};
    /// use roommates::{Error, InvalidFixedCost::*};
    /// use steel_cent::{Money, currency::{USD, EUR}};
    ///
    /// let water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, 100_00),
    /// );
    /// assert!(water_bill.is_ok());
    ///
    /// let bad_water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(EUR, 100_00),
    /// );
    /// assert_eq!(bad_water_bill.unwrap_err(), Error::InvalidFixedCost(MismatchedCurrencies));
    ///
    /// let bad_water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, -100_00),
    /// );
    /// assert_eq!(bad_water_bill.unwrap_err(), Error::InvalidFixedCost(Negative));
    ///
    /// let bad_water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, 200_00),
    /// );
    /// assert_eq!(bad_water_bill.unwrap_err(), Error::InvalidFixedCost(ExceedsAmountDue));
    /// ```
    pub fn new_with_fixed_cost(
        amount_due: Money,
        usage_period: DateInterval,
        fixed_cost: Money,
    ) -> Result<Self, Error> {
        verify_shared_amount(amount_due, fixed_cost)?;
        Ok(Bill {
            amount_due,
            fixed_cost,
            usage_period,
        })
    }

    /// `fixed_cost` represents an explicitly usage-independent portion of the
    /// bill, such as a security deposit
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::Bill};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, 100_00),
    /// ).unwrap();
    /// assert_eq!(water_bill.fixed_cost(), Money::of_minor(USD, 100_00));
    ///
    /// let electric_bill = Bill::new(
    ///     Money::of_minor(USD, 183_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// assert_eq!(electric_bill.fixed_cost(), Money::zero(USD));
    /// ```
    pub fn fixed_cost(&self) -> Money {
        self.fixed_cost
    }

    /// Returns the total cost of the `Bill`
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::Bill};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// assert_eq!(water_bill.amount_due(), Money::of_minor(USD, 83_22));
    /// ```
    pub fn amount_due(&self) -> Money {
        self.amount_due
    }

    /// Returns the interval for which the `Bill` is charging
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::Bill};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// assert_eq!(
    ///     water_bill.usage_period(),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// ```
    pub fn usage_period(&self) -> DateInterval {
        self.usage_period
    }
}

/// A Bill along with the communally shared cost
///
/// `Bill`s can be transformed directly into `SharedBill`s (with the `Bill`s
/// fixed cost becoming the shared amount) but for bills that
/// are usage dependent, there might be an additional implicit fixed cost that
/// can be inferred based on a bill history
pub struct SharedBill {
    bill: Bill,
    shared_amount: Money,
}

impl SharedBill {
    /// Creates a new `SharedBill`
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// let water_bill = SharedBill::new(water_bill, Money::of_minor(USD, 30_00));
    /// assert!(water_bill.is_ok());
    /// ```
    pub fn new(bill: Bill, shared_amount: Money) -> Result<Self, Error> {
        verify_shared_amount(bill.amount_due(), shared_amount)?;
        Ok(SharedBill {
            bill,
            shared_amount,
        })
    }

    /// Returns the amount that should be divided evenly
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// let water_bill = SharedBill::new(water_bill, Money::of_minor(USD, 30_00)).unwrap();
    /// assert_eq!(water_bill.shared_amount(), Money::of_minor(USD, 30_00));
    /// ```
    pub fn shared_amount(&self) -> Money {
        self.shared_amount
    }

    /// Returns the total cost
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// let water_bill = SharedBill::new(water_bill, Money::of_minor(USD, 30_00)).unwrap();
    /// assert_eq!(water_bill.amount_due(), Money::of_minor(USD, 83_22));
    /// ```
    pub fn amount_due(&self) -> Money {
        self.bill.amount_due()
    }

    /// Returns the period of time for which is being billed
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// let water_bill = SharedBill::new(water_bill, Money::of_minor(USD, 30_00)).unwrap();
    /// assert_eq!(
    ///     water_bill.usage_period(),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    /// );
    /// ```
    pub fn usage_period(&self) -> DateInterval {
        self.bill.usage_period()
    }
}

fn verify_shared_amount(amount_due: Money, shared_amount: Money) -> Result<(), Error> {
    let error = if amount_due.currency != shared_amount.currency {
        MismatchedCurrencies
    } else if shared_amount > amount_due {
        ExceedsAmountDue
    } else if shared_amount < Money::zero(shared_amount.currency) {
        Negative
    } else {
        return Ok(());
    };
    Err(Error::InvalidFixedCost(error))
}
