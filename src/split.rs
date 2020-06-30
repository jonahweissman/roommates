use num::rational::Rational;
use std::collections::HashMap;
use steel_cent::Money;

use super::bill::Bill;
use super::roommate::Roommate;

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
pub fn split_bill_list<'a, I>(bills_with_usage_proportions: I) -> HashMap<Roommate, Money>
where
    I: IntoIterator<Item = (&'a Bill, &'a HashMap<Roommate, Rational>)>,
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
        .map(|(bill, usage_proportion)| bill.split(usage_proportion).into_iter())
        .flatten()
        .fold(HashMap::new(), |mut m, (k, v)| {
            let val = m.entry(k).or_insert(Money::zero(currency));
            *val = *val + v;
            m
        })
}

impl Bill {
    fn split(&self, usage_proportion: &HashMap<Roommate, Rational>) -> HashMap<Roommate, Money> {
        let num_roommates = usage_proportion.len() as i64;
        usage_proportion
            .keys()
            .map(|roommate| {
                (
                    roommate.clone(),
                    (self.shared_cost() / num_roommates)
                        + (self.amount_due() - self.shared_cost())
                            .mul_rational(*usage_proportion.get(roommate).unwrap()),
                )
            })
            .collect()
    }
}

trait MulRational {
    fn mul_rational(&self, r: Rational) -> Self;
}

impl MulRational for Money {
    fn mul_rational(&self, r: Rational) -> Self {
        self.checked_mul(*r.numer() as i64)
            .expect("overflow")
            .checked_div(*r.denom() as i64)
            .expect("overflow")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::responsibility::interval::DateInterval;
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
        let roomies = vec![
            Roommate::new(String::from("bob")),
            Roommate::new(String::from("joe")),
        ];
        let usage_proportions = vec![Rational::new(1, 4), Rational::new(3, 4)];
        let usage: HashMap<Roommate, Rational> =
            roomies.into_iter().zip(usage_proportions).collect();
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bill = new_bill(total, shared_cost);
        let share = &bill.split(&usage);
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        assert_eq!(bob_share, shared_cost / 2 + (total - shared_cost) * 0.25);
    }

    #[test]
    fn list_of_bills() {
        let roomies = vec![
            Roommate::new(String::from("bob")),
            Roommate::new(String::from("joe")),
        ];
        let usage_proportions = vec![Rational::new(1, 4), Rational::new(3, 4)];
        let usage: HashMap<Roommate, Rational> =
            roomies.into_iter().zip(usage_proportions).collect();
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bills = vec![
            new_bill(total, shared_cost),
            new_bill(total * 2, shared_cost * 2),
        ];
        let share = split_bill_list(bills.iter().map(|bill| (bill, &usage)).collect::<Vec<_>>());
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        let expected = shared_cost.checked_mul_f(1.5).unwrap()
            + (total - shared_cost).checked_mul_f(0.25).unwrap() * 3;
        assert_eq!(bob_share, expected);
        assert_eq!(
            bills[0].amount_due(),
            bills[0].amount_due(),
            "bills shouldn't be consumed"
        );
        assert!(usage == usage);
    }

    #[test]
    fn list_of_zero_valued_bills() {
        let roomies = vec![
            Roommate::new(String::from("bob")),
            Roommate::new(String::from("joe")),
        ];
        let usage_proportions = vec![Rational::new(1, 4), Rational::new(3, 4)];
        let usage: HashMap<Roommate, Rational> =
            roomies.into_iter().zip(usage_proportions).collect();
        let total = Money::of_major_minor(USD, 0, 0);
        let shared_cost = Money::of_major_minor(USD, 0, 0);
        let bills = vec![
            new_bill(total, shared_cost),
            new_bill(total * 2, shared_cost * 2),
        ];
        let bill_list: Vec<_> = bills.iter().map(|bill| (bill, &usage)).collect();
        let share = split_bill_list(bill_list);
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        let expected = Money::zero(USD);
        assert_eq!(bob_share, expected);
        assert_eq!(
            bills[0].amount_due(),
            bills[0].amount_due(),
            "bills shouldn't be consumed"
        );
    }
}
