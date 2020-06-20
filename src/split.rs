use num::rational::Rational;
use std::collections::HashMap;
use steel_cent::currency::USD;
use steel_cent::Money;

use super::roommate::Roommate;
use super::bill::Bill;

impl Bill {
    fn split(
        &self,
        usage_proportion: &HashMap<Roommate, Rational>,
    ) -> HashMap<Roommate, Money> {
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

pub fn split_bill_list(
    bills_with_usage_proportions: &Vec<(&Bill, &HashMap<Roommate, Rational>)>,
) -> HashMap<Roommate, Money> {
    bills_with_usage_proportions
        .iter()
        .map(|(bill, usage_proportion)| bill.split(usage_proportion).into_iter())
        .flatten()
        .fold(HashMap::new(), |mut m, (k, v)| {
            let val = m.entry(k).or_insert(Money::zero(USD));
            *val = *val + v;
            m
        })
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
        let bill = Bill::new(total, Some(shared_cost));
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
            Bill::new(total, Some(shared_cost)),
            Bill::new(total * 2, Some(shared_cost * 2)),
        ];
        let share = split_bill_list(&bills.iter().map(|bill| (bill, &usage)).collect());
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        let expected = shared_cost.checked_mul_f(1.5).unwrap()
            + (total - shared_cost).checked_mul_f(0.25).unwrap() * 3;
        assert_eq!(bob_share, expected);
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
            Bill::new(total, Some(shared_cost)),
            Bill::new(total * 2, Some(shared_cost * 2)),
        ];
        let share = split_bill_list(&bills.iter().map(|bill| (bill, &usage)).collect());
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        let expected = Money::zero(USD);
        assert_eq!(bob_share, expected);
    }
}
