use num::rational::Rational;
use std::fmt;
use steel_cent::Money;

use super::bill::Bill;
use super::responsibility;
use super::interval::ResponsibilityInterval;
use super::roommate::{Roommate, RoommateGroup};

pub struct Invoice {
    to: Roommate,
    total: Money,
    components: Vec<InvoiceComponent>,
}

struct InvoiceComponent {
    bill: String,
    shared_cost: Money,
    intervals: Vec<ResponsibilityInterval>,
    responsibility_proportion: Rational,
    personal_cost: Money,
}

impl RoommateGroup {
    pub fn generate_invoices<'a, I, J>(
        &self,
        bills: J,
        responsibility_intervals: Vec<ResponsibilityInterval>,
    ) -> Vec<Invoice>
    where
        J: IntoIterator<Item = (&'a str, (Bill, Option<f64>), I)>,
        I: IntoIterator<Item = (Bill, Option<f64>)>,
    {
        let bill_list = bills
            .into_iter()
            .map(|(label, (mut current_bill, current_ti), history_with_ti)| {
                let history = history_with_ti
                    .into_iter()
                    .map(|(mut bill, temperature_index)| {
                        bill.add_notes(&responsibility_intervals, temperature_index);
                        bill
                    })
                    .collect::<Vec<_>>();
                current_bill.add_notes(&responsibility_intervals, current_ti);
                current_bill.calculate_shared_cost(history.iter().collect::<Vec<_>>());
                let split = self
                    .individual_responsibilities(&responsibility_intervals, current_bill.period());
                (current_bill, split)
            })
            .collect::<Vec<_>>();
        self.split_bill_list(bill_list.iter().map(|(b, s)| (b, s)).collect::<Vec<_>>())
            .into_iter()
            .map(|(to, total)| Invoice {
                to,
                total,
                components: vec![],
            })
            .collect()
    }
}

impl Bill {
    fn add_notes(
        &mut self,
        responsibility_intervals: &Vec<ResponsibilityInterval>,
        temperature_index: Option<f64>,
    ) {
        let billing_period = self.period().clone();
        self.usage_notes_mut()
            .update_average_occupancy(responsibility::proportion_of_interval(
                responsibility_intervals,
                &billing_period,
            ));
        if let Some(ti) = temperature_index {
            self.usage_notes_mut().update_temperature_index(ti);
        }
    }
}

impl fmt::Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} owes {}", self.to, self.total)
    }
}
