use steel_cent::Money;

use super::responsibility::interval::DateInterval;

pub struct Bill {
    amount_due: Money,
    shared_cost: Money,
    period: DateInterval,
}

impl Bill {
    pub fn new(amount_due: Money, shared_cost: Option<Money>, period: DateInterval) -> Self {
        let shared_cost = shared_cost.unwrap_or(Money::zero(amount_due.currency));
        assert!(shared_cost <= amount_due);
        Bill {
            amount_due,
            shared_cost,
            period,
        }
    }

    pub fn amount_due(&self) -> &Money {
        &self.amount_due
    }

    pub fn shared_cost(&self) -> &Money {
        &self.shared_cost
    }
}
