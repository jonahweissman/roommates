use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;

/// Someone living in the housing unit
///
/// A wrapper around the name of someone financially responsible for charges
#[derive(Hash, PartialEq, Clone, Debug)]
pub struct Roommate(String);

impl Roommate {
    /// Creates a new `Roommate`
    ///
    /// # Examples
    /// ```
    /// use roommates::Roommate;
    ///
    /// let bob = Roommate::new("Bob");
    /// assert_eq!(format!("{}", bob), "Bob");
    /// ```
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

/// A collection of [`Roommate`]s
///
/// [`Roommate`]: struct.Roommate.html
pub struct RoommateGroup(HashSet<Roommate>);

impl RoommateGroup {
    /// Returns the number of `Roommate`s in the `RoommateGroup`
    ///
    /// # Examples
    /// ```
    /// use roommates::RoommateGroup;
    ///
    /// let group: RoommateGroup = vec!["Bob", "Joe"].into_iter().collect();
    /// assert_eq!(group.count(), 2);
    /// ```
    pub fn count(&self) -> u32 {
        self.0.len() as u32
    }

    /// Returns an iterator that visits each `Roommate` in the group
    pub fn iter(&self) -> impl Iterator<Item = &Roommate> {
        self.0.iter()
    }

    /// Returns a reference to a roommate with the given name
    ///
    /// # Examples
    /// ```
    /// use roommates::{Roommate, RoommateGroup};
    ///
    /// let group: RoommateGroup = vec!["Bob", "Joe"].into_iter().collect();
    /// assert_eq!(group.borrow_by_name("Bob"), Some(&Roommate::new("Bob")));
    /// assert_eq!(group.borrow_by_name("Steve"), None);
    /// ```
    pub fn borrow_by_name(&self, name: &str) -> Option<&Roommate> {
        self.0.iter().find(|Roommate(n)| n == name)
    }
}

impl<'a> FromIterator<&'a str> for RoommateGroup {
    fn from_iter<I: IntoIterator<Item = &'a str>>(names: I) -> Self {
        RoommateGroup(names.into_iter().map(|n| Roommate::new(n)).collect())
    }
}
