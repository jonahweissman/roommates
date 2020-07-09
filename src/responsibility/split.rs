use num::rational::Ratio;
use std::collections::{hash_map::Iter, HashMap};

use super::super::roommate::{Roommate, RoommateGroup};

pub struct ResponsibilitySplit(HashMap<Roommate, Ratio<u32>>);

impl ResponsibilitySplit {
    pub fn iter(&self) -> Iter<Roommate, Ratio<u32>> {
        self.0.iter()
    }
}

impl RoommateGroup {
    pub fn build_split(&self, map: HashMap<Roommate, Ratio<u32>>) -> ResponsibilitySplit {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::iter;

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
            rs.iter().collect::<HashSet<_>>(),
            roomies
                .iter()
                .zip(iter::repeat(&Ratio::new(1, 3)).take(3))
                .collect::<HashSet<_>>(),
        );
    }
}
