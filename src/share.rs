use itertools::Itertools;
use std::collections::HashMap;
use steel_cent::currency::USD;
use steel_cent::Money;

use super::roommate::Roommate;

struct Bill {
    amount_due: Money,
    shared_cost: Money,
}

impl Bill {
    pub fn new(amount_due: Money, shared_cost: Option<Money>) -> Self {
        let shared_cost = shared_cost.unwrap_or(Money::zero(USD));
        assert!(shared_cost <= amount_due);
        Bill {
            amount_due,
            shared_cost,
        }
    }
}

fn share_bill(bill: &Bill, usage_proportion: &HashMap<Roommate, f64>) -> HashMap<Roommate, Money> {
    let num_roommates = usage_proportion.len() as i64;
    usage_proportion
        .keys()
        .map(|roommate| {
            (
                roommate.clone(),
                (bill.shared_cost / num_roommates)
                    + ((bill.amount_due - bill.shared_cost)
                        .checked_mul_f(*usage_proportion.get(roommate).unwrap())
                        .expect("Bill proportion went out of bounds")),
            )
        })
        .collect()
}

fn share_bill_list(
    bills_with_usage_proportions: &Vec<(&Bill, &HashMap<Roommate, f64>)>,
) -> HashMap<Roommate, Money> {
    bills_with_usage_proportions
        .iter()
        .map(|(bill, usage_proportion)| share_bill(bill, usage_proportion).into_iter())
        .flatten()
        .into_group_map()
        .into_iter()
        .map(|(k, v)| (k.clone(), v.into_iter().fold1(|a, x| a + x).unwrap()))
        .collect()
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
        let usage: HashMap<Roommate, f64> = roomies.into_iter().zip(vec![0.25, 0.75]).collect();
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bill = Bill::new(total, Some(shared_cost));
        let share = share_bill(&bill, &usage);
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        assert_eq!(bob_share, shared_cost / 2 + (total - shared_cost) * 0.25);
    }

    #[test]
    fn list_of_bills() {
        let roomies = vec![
            Roommate::new(String::from("bob")),
            Roommate::new(String::from("joe")),
        ];
        let usage: HashMap<Roommate, f64> = roomies.into_iter().zip(vec![0.25, 0.75]).collect();
        let total = Money::of_major_minor(USD, 99, 99);
        let shared_cost = Money::of_major_minor(USD, 35, 46);
        let bills = vec![
            Bill::new(total, Some(shared_cost)),
            Bill::new(total * 2, Some(shared_cost * 2)),
        ];
        let share = share_bill_list(&bills.iter().map(|bill| (bill, &usage)).collect());
        let bob_share = *share.get(&Roommate::new(String::from("bob"))).unwrap();
        let expected = shared_cost.checked_mul_f(1.5).unwrap()
            + (total - shared_cost).checked_mul_f(0.25).unwrap() * 3;
        assert_eq!(bob_share, expected);
    }
}
