use num::rational::Ratio;
use std::cmp::{max, min};
use std::collections::HashMap;
use steel_cent::Money;

use super::bill::Bill;
use super::responsibility::split::ResponsibilitySplit;
use super::roommate::{Roommate, RoommateGroup};

impl RoommateGroup {
    /// Takes a vector (or other collection that can be turned into an iter)
    /// of [`Bill`]s with corresponding maps for each each Bill of
    /// how much each roommate is personally responsible for and then outputs
    /// a new HashMap that accumulates all
    ///
    /// Consumes the collection, but not the bills and hashmaps.
    ///
    /// ## Panics
    /// Panics if bills are not all in the same currency.
    /// Panics if the list is empty.
    pub fn split_bill_list<'a, I>(
        &self,
        bills_with_usage_proportions: I,
    ) -> HashMap<Roommate, Money>
    where
        I: IntoIterator<Item = (&'a Bill, &'a ResponsibilitySplit)>,
    {
        let mut bills_with_usage_proportions = bills_with_usage_proportions.into_iter().peekable();
        let currency = bills_with_usage_proportions
            .peek()
            .expect("must include at least one bill")
            .0
            .amount_due()
            .currency;

        bills_with_usage_proportions
            .inspect(|(bill, _)| {
                assert!(
                    bill.amount_due().currency == currency,
                    "all bills must have the same currency"
                )
            })
            .map(|(bill, usage_proportion)| self.split(bill, usage_proportion).into_iter())
            .flatten()
            .fold(HashMap::new(), |mut m, (k, v)| {
                let val = m.entry(k).or_insert(Money::zero(currency));
                *val = *val + v;
                m
            })
    }

    fn split(
        &self,
        bill: &Bill,
        usage_proportion: &ResponsibilitySplit,
    ) -> HashMap<Roommate, Money> {
        usage_proportion
            .iter()
            .map(|(roommate, share)| (roommate.clone(), self.divide(bill, share)))
            .collect()
    }

    fn divide(&self, bill: &Bill, personally_responsible: &Ratio<u32>) -> Money {
        (Money::of_minor(
            bill.amount_due().currency,
            min(
                bill.amount_due().minor_amount(),
                bill.shared_cost().minor_amount(),
            ),
        ) / self.len() as i64)
            + Money::of_minor(
                bill.amount_due().currency,
                max(0, (bill.amount_due() - bill.shared_cost()).minor_amount()),
            )
            .mul_rational(*personally_responsible)
    }
}

trait MulRatiou32 {
    fn mul_rational(&self, r: Ratio<u32>) -> Self;
}

impl MulRatiou32 for Money {
    fn mul_rational(&self, r: Ratio<u32>) -> Self {
        self.checked_mul(*r.numer() as i64)
            .expect("overflow")
            .checked_div(*r.denom() as i64)
            .expect("overflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval::DateInterval;
    use chrono::naive::NaiveDate;
    use steel_cent::currency::USD;

    fn new_bill(total: Money, shared: Money) -> Bill {
        // we only care about the total and shared for these tests
        let start = NaiveDate::parse_from_str("01/02/20", "%D").unwrap();
        let end = NaiveDate::parse_from_str("02/02/20", "%D").unwrap();
        Bill::new(total, Some(shared), DateInterval::new(start, end))
    }

    #[test]
    fn regular_bill() {
        let (bob, joe) = (Roommate::new("bob"), Roommate::new("joe"));
        let roomies = RoommateGroup::new(vec![&bob, &joe]);
        let usage_proportions = vec![Ratio::new(1, 4), Ratio::new(3, 4)];
        let split = roomies.build_split(
            vec![bob.clone(), joe.clone()]
                .into_iter()
                .zip(usage_proportions)
                .collect(),
        );
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bill = new_bill(total, shared_cost);
        let share = &roomies.split(&bill, &split);
        let bob_share = *share.get(&bob).unwrap();
        assert_eq!(bob_share, shared_cost / 2 + (total - shared_cost) * 0.25);
    }

    #[test]
    fn list_of_bills() {
        let (bob, joe) = (Roommate::new("bob"), Roommate::new("joe"));
        let roomies = RoommateGroup::new(vec![&bob, &joe]);
        let usage_proportions = vec![Ratio::new(1, 4), Ratio::new(3, 4)];
        let split = roomies.build_split(
            vec![bob.clone(), joe.clone()]
                .into_iter()
                .zip(usage_proportions)
                .collect(),
        );
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bills = vec![
            new_bill(total, shared_cost),
            new_bill(total * 2, shared_cost * 2),
        ];
        let share =
            roomies.split_bill_list(bills.iter().map(|bill| (bill, &split)).collect::<Vec<_>>());
        let bob_share = *share.get(&bob).unwrap();
        let expected = shared_cost.checked_mul_f(1.5).unwrap()
            + (total - shared_cost).checked_mul_f(0.25).unwrap() * 3;
        assert_eq!(bob_share, expected);
        assert_eq!(
            bills[0].amount_due(),
            bills[0].amount_due(),
            "bills shouldn't be consumed"
        );
    }

    #[test]
    fn list_of_zero_valued_bills() {
        let (bob, joe) = (Roommate::new("bob"), Roommate::new("joe"));
        let roomies = RoommateGroup::new(vec![&bob, &joe]);
        let usage_proportions = vec![Ratio::new(1, 4), Ratio::new(3, 4)];
        let split = roomies.build_split(
            vec![bob.clone(), joe.clone()]
                .into_iter()
                .zip(usage_proportions)
                .collect(),
        );
        let total = Money::of_major_minor(USD, 0, 0);
        let shared_cost = Money::of_major_minor(USD, 0, 0);
        let bills = vec![
            new_bill(total, shared_cost),
            new_bill(total * 2, shared_cost * 2),
        ];
        let bill_list: Vec<_> = bills.iter().map(|bill| (bill, &split)).collect();
        let share = roomies.split_bill_list(bill_list);
        let bob_share = *share.get(&bob).unwrap();
        let expected = Money::zero(USD);
        assert_eq!(bob_share, expected);
        assert_eq!(
            bills[0].amount_due(),
            bills[0].amount_due(),
            "bills shouldn't be consumed"
        );
    }

    #[test]
    fn no_reponsibilities() {
        let (bob, joe) = (Roommate::new("bob"), Roommate::new("joe"));
        let roomies = RoommateGroup::new(vec![&bob, &joe]);
        let usage_proportions = vec![Ratio::from_integer(0), Ratio::from_integer(0)];
        let split = roomies.build_split(
            vec![bob.clone(), joe.clone()]
                .into_iter()
                .zip(usage_proportions)
                .collect(),
        );
        let total = Money::of_major_minor(USD, 30, 0);
        let shared_cost = Money::of_major_minor(USD, 25, 0);
        let bills = vec![new_bill(total, shared_cost)];
        let bill_list: Vec<_> = bills.iter().map(|bill| (bill, &split)).collect();
        let share = roomies.split_bill_list(bill_list);
        let bob_share = *share.get(&bob).unwrap();
        let joe_share = *share.get(&joe).unwrap();
        assert_eq!(bob_share, joe_share);
        assert_eq!(bob_share + joe_share, total);
    }

    #[test]
    fn shared_cost_more_than_bill() {
        let (bob, joe) = (Roommate::new("bob"), Roommate::new("joe"));
        let roomies = RoommateGroup::new(vec![&bob, &joe]);
        let usage_proportions = vec![Ratio::new(1, 2), Ratio::new(1, 2)];
        let split = roomies.build_split(
            vec![bob.clone(), joe.clone()]
                .into_iter()
                .zip(usage_proportions)
                .collect(),
        );
        let total = Money::of_major_minor(USD, 30, 0);
        let shared_cost = Money::of_major_minor(USD, 35, 0);
        let bills = vec![new_bill(total, shared_cost)];
        let bill_list: Vec<_> = bills.iter().map(|bill| (bill, &split)).collect();
        let share = roomies.split_bill_list(bill_list);
        let bob_share = *share.get(&bob).unwrap();
        let joe_share = *share.get(&joe).unwrap();
        assert_eq!(bob_share, joe_share);
        assert_eq!(bob_share + joe_share, total);
    }
}
