use itertools::Itertools;
use num::rational::Ratio;
use std::collections::HashMap;
use std::fmt;
use steel_cent::Money;

use super::bill::Bill;
use super::interval::ResponsibilityInterval;
use super::responsibility;
use super::roommate::{Roommate, RoommateGroup};

pub struct Invoice {
    to: Roommate,
    total: Money,
    components: Vec<InvoiceComponent>,
}

struct InvoiceComponent {
    label: String,
    bill: Bill,
    responsibility_proportion: Ratio<u32>,
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
        let mut invoice_components: HashMap<Roommate, Vec<InvoiceComponent>> = HashMap::new();
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
                if current_bill.shared_cost().is_none() {
                    current_bill.calculate_shared_cost(history.iter().collect::<Vec<_>>());
                }
                let split = self
                    .individual_responsibilities(&responsibility_intervals, current_bill.period());
                for (roommate, share) in split.iter() {
                    invoice_components
                        .entry(roommate.clone())
                        .or_insert(vec![])
                        .push(InvoiceComponent {
                            label: String::from(label),
                            responsibility_proportion: share.clone(),
                            bill: current_bill.clone(),
                        })
                }
                (current_bill, split)
            })
            .collect::<Vec<_>>();
        self.split_bill_list(bill_list.iter().map(|(b, s)| (b, s)).collect::<Vec<_>>())
            .into_iter()
            .map(|(to, total)| {
                let components = invoice_components.remove(&to).unwrap();
                Invoice {
                    to,
                    total,
                    components,
                }
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
        write!(
            f,
            "{} owes {}\n{}",
            self.to,
            self.total,
            self.components.iter().join("\n")
        )
    }
}

impl fmt::Display for InvoiceComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\t{} of the responsibility for the {} {} bill",
            self.responsibility_proportion,
            self.bill.amount_due(),
            self.label
        )
    }
}
