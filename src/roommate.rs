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
    pub fn new<'a, I>(roommates: I) -> Self
    where
        I: IntoIterator<Item = &'a Roommate>,
    {
        RoommateGroup(roommates.into_iter().cloned().collect())
    }

    pub fn from_strs(names: Vec<&str>) -> Self {
        RoommateGroup(names.into_iter().map(|n| Roommate::new(n)).collect())
    }

    pub fn count(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn set(&self) -> &HashSet<Roommate> {
        &self.0
    }

    pub fn borrow_by_name(&self, name: &str) -> Option<&Roommate> {
        self.0.iter().find(|r| &r.0[..] == name)
    }
}
