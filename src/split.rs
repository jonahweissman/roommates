use num::rational::Ratio;
use std::collections::HashMap;
use steel_cent::Money;

use super::bill::SharedBill;
use super::interval::{DateInterval, ResponsibilityRecord};
use super::roommate::{Roommate, RoommateGroup};

impl RoommateGroup {
    /// Returns the proportion of the total cost that each contributing party
    /// is responsible for
    pub fn individual_responsibilities(
        &self,
        responsibility_intervals: &ResponsibilityRecord,
        billing_period: DateInterval,
    ) -> ResponsibilitySplit {
        let map: HashMap<Roommate, Ratio<u32>> = self
            .set()
            .iter()
            .map(|roommate| {
                (
                    roommate.clone(),
                    responsibility_intervals.roommate_responsibility(roommate, billing_period),
                )
            })
            .collect();
        self.build_split(map)
    }

    /// Takes a vector (or other collection that can be turned into an iter)
    /// of [`Bill`]s with corresponding maps for each each Bill of
    /// how much each roommate is personally responsible for and then outputs
    /// a new HashMap that accumulates all
    ///
    /// ## Panics
    /// Panics if bills are not all in the same currency.
    /// Panics if the list is empty.
    pub fn split_bill_list<'a, I>(
        &self,
        bills_with_usage_proportions: I,
    ) -> HashMap<Roommate, Money>
    where
        I: IntoIterator<Item = (&'a SharedBill, &'a ResponsibilitySplit)>,
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
                let val = m.entry(k).or_insert_with(|| Money::zero(currency));
                *val = *val + v;
                m
            })
    }

    fn split(
        &self,
        bill: &SharedBill,
        usage_proportion: &ResponsibilitySplit,
    ) -> HashMap<Roommate, Money> {
        usage_proportion
            .hash_map()
            .into_iter()
            .map(|(roommate, share)| (roommate.clone(), self.divide(bill, share)))
            .collect()
    }

    fn divide(&self, bill: &SharedBill, personally_responsible: Ratio<u32>) -> Money {
        (Money::of_minor(
            bill.amount_due().currency,
            bill.shared_amount().minor_amount(),
        ) / self.count() as i64)
            + Money::of_minor(
                bill.amount_due().currency,
                (bill.amount_due() - bill.shared_amount()).minor_amount(),
            )
            .mul_rational(personally_responsible)
    }

    fn build_split(&self, map: HashMap<Roommate, Ratio<u32>>) -> ResponsibilitySplit {
        let sum = map.values().sum::<Ratio<u32>>();
        let all_roommates = self.set().iter();
        let map: HashMap<_, _> = if sum == Ratio::from_integer(1) {
            all_roommates
                .map(|r| {
                    (
                        r.clone(),
                        *map.get(r)
                            .unwrap_or_else(|| panic!("roommate not in RoommateGroup {}", r)),
                    )
                })
                .collect()
        } else if sum == Ratio::from_integer(0) {
            all_roommates
                .map(|r| (r.clone(), Ratio::new(1u32, self.count())))
                .collect()
        } else {
            panic!("sum must be 1 or 0")
        };
        ResponsibilitySplit(map)
    }
}

impl ResponsibilityRecord<'_> {
    fn roommate_responsibility(
        &self,
        roommate: &Roommate,
        billing_period: DateInterval,
    ) -> Ratio<u32> {
        let roommate_intervals = self
            .iter()
            .filter(|i| i.roommate() == roommate)
            .cloned()
            .collect::<ResponsibilityRecord>();
        let total_cost = self.occupancy_over(billing_period);
        if total_cost == 0 {
            return Ratio::from_integer(0);
        }
        Ratio::new(
            roommate_intervals.occupancy_over(billing_period),
            total_cost,
        )
    }
}

pub struct ResponsibilitySplit(HashMap<Roommate, Ratio<u32>>);

impl ResponsibilitySplit {
    pub fn hash_map(&self) -> HashMap<&Roommate, Ratio<u32>> {
        self.0.iter().map(|(r, s)| (r, *s)).collect()
    }
}

trait MulRatio<T> {
    fn mul_rational(&self, r: Ratio<T>) -> Self;
}

impl MulRatio<u32> for Money {
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
    use crate::bill::Bill;
    use crate::interval::{DateInterval, ResponsibilityInterval};
    use std::collections::HashSet;
    use std::iter;
    use steel_cent::currency::USD;

    fn build_rs(rg: RoommateGroup, pairs: Vec<(&str, u32, u32)>) -> ResponsibilitySplit {
        rg.build_split(
            pairs
                .into_iter()
                .map(|(name, n, d)| (Roommate::new(name), Ratio::new(n, d)))
                .collect(),
        )
    }

    #[test]
    #[should_panic]
    fn sum_over_one() {
        let roomies = vec!["a", "b", "c"]
            .into_iter()
            .map(|n| Roommate::new(n))
            .collect::<Vec<_>>();
        let rg = RoommateGroup::new(roomies.iter().collect::<Vec<_>>());
        let _rs = build_rs(rg, vec![("a", 2, 3), ("b", 1, 3), ("c", 1, 3)]);
    }

    #[test]
    fn empty_list() {
        let roomies = vec!["a", "b", "c"]
            .into_iter()
            .map(|n| Roommate::new(n))
            .collect::<Vec<_>>();
        let rg = RoommateGroup::new(roomies.iter().collect::<Vec<_>>());
        let rs = build_rs(rg, vec![]);
        assert_eq!(
            rs.hash_map().into_iter().collect::<HashSet<_>>(),
            roomies
                .iter()
                .zip(iter::repeat(Ratio::new(1, 3)).take(3))
                .collect::<HashSet<_>>(),
        );
    }

    #[test]
    fn partial_interval_with_weights_responsibilities() {
        let start = (2020, 1, 10);
        let end = (2020, 1, 20);
        let me = Roommate::new("me");
        let someone = Roommate::new("someone");
        let intervals = vec![
            ResponsibilityInterval::new(&me, DateInterval::new((2020, 1, 18), end).unwrap(), 1),
            ResponsibilityInterval::new(
                &someone,
                DateInterval::new(start, (2020, 1, 13)).unwrap(),
                3,
            ),
        ];
        let record = intervals.iter().cloned().collect::<ResponsibilityRecord>();
        let group = RoommateGroup::new(vec![&Roommate::new("me"), &Roommate::new("someone")]);
        let split =
            group.individual_responsibilities(&record, DateInterval::new(start, end).unwrap());
        let table: HashMap<_, _> = split.hash_map();
        assert_eq!(
            *table.get(&intervals[0].roommate()).unwrap(),
            Ratio::new(2 * 3, 4 * 4 + 2 * 3),
        );
        assert_eq!(
            table.values().cloned().sum::<Ratio<u32>>(),
            Ratio::from_integer(1),
        );
    }

    #[test]
    fn no_overlap_between_billing_period_and_intervals() {
        let start = (2020, 1, 2);
        let end = (2020, 2, 2);
        let me = Roommate::new("me");
        let intervals = vec![ResponsibilityInterval::new(
            &me,
            DateInterval::new((2019, 1, 2), (2019, 2, 2)).unwrap(),
            0,
        )];
        let record = intervals.iter().cloned().collect::<ResponsibilityRecord>();
        let billing_period = DateInterval::new(start, end).unwrap();
        let group = RoommateGroup::new(vec![&Roommate::new("me"), &Roommate::new("someone")]);
        assert_eq!(record.occupancy_over(billing_period), 0);
        assert_eq!(
            **group
                .individual_responsibilities(&record, billing_period)
                .hash_map()
                .iter()
                .collect::<HashMap<_, _>>()
                .get(&intervals[0].roommate())
                .unwrap(),
            Ratio::new(1, 2)
        );
    }

    fn new_bill(total: Money, shared: Money) -> SharedBill {
        SharedBill::new(
            Bill::new(
                total,
                DateInterval::new((2020, 1, 2), (2020, 2, 2)).unwrap(),
                None,
            ),
            shared,
        )
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
    #[should_panic]
    fn shared_cost_more_than_bill() {
        let total = Money::of_major_minor(USD, 30, 0);
        let shared_cost = Money::of_major_minor(USD, 35, 0);
        vec![new_bill(total, shared_cost)];
    }

    #[test]
    #[should_panic]
    fn shared_cost_less_than_zero() {
        let total = Money::of_major_minor(USD, 30, 0);
        let shared_cost = Money::of_major_minor(USD, -1, 0);
        vec![new_bill(total, shared_cost)];
    }

    #[test]
    #[ignore]
    fn rounding_issue_everyone_pays_the_same() {
        let r = vec!["a", "b", "c"]
            .into_iter()
            .map(|n| Roommate::new(n))
            .collect::<Vec<_>>();
        let roomies = RoommateGroup::new(r.iter());
        let usage_proportions = vec![0, 0, 0]
            .into_iter()
            .map(|p| Ratio::from_integer(p))
            .collect::<Vec<_>>();
        let split = roomies.build_split(r.iter().cloned().zip(usage_proportions).collect());
        let total = Money::of_major_minor(USD, 20, 00);
        let shared_cost = Money::of_major_minor(USD, 10, 00);
        let bills = vec![new_bill(total, shared_cost)];
        let bill_list: Vec<_> = bills.iter().map(|bill| (bill, &split)).collect();
        let share = roomies.split_bill_list(bill_list);
        let actual_total = share.values().fold(Money::zero(USD), |a, x| a + x);
        assert_eq!(total, actual_total);
    }
}
