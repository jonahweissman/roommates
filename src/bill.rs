use steel_cent::Money;
use steel_cent::currency::USD;

use super::responsibility::interval::NaiveDateInterval;

pub struct Bill {
    amount_due: Money,
    shared_cost: Money,
    interval: Option<NaiveDateInterval>,
}

impl Bill {
    pub fn new(amount_due: Money, shared_cost: Option<Money>) -> Self {
        let shared_cost = shared_cost.unwrap_or(Money::zero(USD));
        assert!(shared_cost <= amount_due);
        Bill {
            amount_due,
            shared_cost,
            interval: None,
        }
    }

    pub fn amount_due(&self) -> &Money {
        &self.amount_due
    }

    pub fn shared_cost(&self) -> &Money {
        &self.shared_cost
    }
}
