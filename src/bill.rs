use steel_cent::Money;

use super::interval::DateInterval;

#[derive(Debug, Clone)]
pub struct Bill {
    amount_due: Money,
    fixed_cost: Money,
    period: DateInterval,
}

impl Bill {
    pub fn new(amount_due: Money, period: DateInterval, fixed_cost: Option<Money>) -> Self {
        let fixed_cost = match fixed_cost {
            Some(fixed_cost) => {
                verify_shared_amount(amount_due, fixed_cost).expect("invalid fixed cost");
                fixed_cost
            }
            None => Money::zero(amount_due.currency),
        };
        Bill {
            amount_due,
            fixed_cost,
            period,
        }
    }

    pub fn fixed_cost(&self) -> Money {
        self.fixed_cost
    }

    pub fn amount_due(&self) -> Money {
        self.amount_due
    }

    pub fn period(&self) -> DateInterval {
        self.period
    }
}

pub struct SharedBill {
    bill: Bill,
    shared_amount: Money,
}

impl SharedBill {
    pub fn new(bill: Bill, shared_amount: Money) -> Self {
        verify_shared_amount(bill.amount_due(), shared_amount).expect("invalid shared amount");
        SharedBill {
            bill,
            shared_amount,
        }
    }

    pub fn shared_amount(&self) -> Money {
        self.shared_amount
    }

    pub fn amount_due(&self) -> Money {
        self.bill.amount_due()
    }

    pub fn period(&self) -> DateInterval {
        self.bill.period()
    }
}

fn verify_shared_amount(amount_due: Money, shared_amount: Money) -> Result<(), ()> {
    assert_eq!(
        amount_due.currency, shared_amount.currency,
        "cannot share in a different currency than the bill"
    );
    assert!(
        shared_amount <= amount_due,
        "cannot share more than the value of the bill"
    );
    assert!(
        shared_amount >= Money::zero(shared_amount.currency),
        "cannot share a negative amount"
    );
    Ok(())
}
