use std::fmt;

#[derive(Hash, PartialEq, Clone)]
pub struct Roommate(String);

impl Roommate {
    pub fn new(name: String) -> Self {
        Roommate(name)
    }
}

impl fmt::Display for Roommate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for Roommate {}
