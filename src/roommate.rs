use std::collections::HashSet;
use std::fmt;

#[derive(Hash, PartialEq, Clone, Debug)]
pub struct Roommate(String);

impl Roommate {
    pub fn new(name: &str) -> Self {
        Roommate(String::from(name))
    }
}

impl fmt::Display for Roommate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for Roommate {}

pub struct RoommateGroup(HashSet<Roommate>);

impl RoommateGroup {
    pub fn new(roommates: Vec<&Roommate>) -> Self {
        RoommateGroup(roommates.into_iter().map(|r| r.clone()).collect())
    }

    pub fn from_strs(names: Vec<&str>) -> Self {
        RoommateGroup(names.into_iter().map(|n| Roommate::new(n)).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set(&self) -> &HashSet<Roommate> {
        &self.0
    }
}
