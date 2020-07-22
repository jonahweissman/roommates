use itertools::Itertools;
use num::rational::Ratio;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use steel_cent::Money;

use super::bill::{Bill, SharedBill};
use super::interval::ResponsibilityRecord;
use super::roommate::{Roommate, RoommateGroup};

pub struct Invoice {
    to: Roommate,
    total: Money,
    components: Vec<InvoiceComponent>,
}

struct InvoiceComponent {
    label: String,
    amount_due: Money,
    shared_amount: Money,
    responsibility_proportion: Ratio<u32>,
}

pub enum SharingData<I: IntoIterator<Item = (Bill, Option<f64>)>> {
    Fixed(Bill),
    Variable((Bill, Option<f64>), I),
}

impl RoommateGroup {
    pub fn generate_invoices<'a, I, J>(
        &self,
        bills: J,
        responsibility_intervals: &ResponsibilityRecord,
    ) -> Vec<Invoice>
    where
        J: IntoIterator<Item = (&'a str, SharingData<I>)>,
        I: IntoIterator<Item = (Bill, Option<f64>)>,
    {
        let mut invoice_components: HashMap<Roommate, Vec<InvoiceComponent>> = HashMap::new();
        let bill_list = bills
            .into_iter()
            .map(|(label, sharing_data)| match sharing_data {
                SharingData::Variable(current_bill, history) => {
                    estimate_shared_bills(label, current_bill, history, responsibility_intervals)
                        .expect("estimating failed")
                }
                SharingData::Fixed(bill) => (
                    label,
                    SharedBill::from_fixed(
                        Bill::new_with_fixed_cost(
                            bill.amount_due(),
                            bill.usage_period(),
                            bill.amount_due(),
                        )
                        .expect("invalid fixed cost"),
                    ),
                ),
            })
            .map(|(label, shared_bill)| {
                let split = self.individual_responsibilities(
                    responsibility_intervals,
                    shared_bill.usage_period(),
                );
                for (roommate, share) in split.hash_map().into_iter() {
                    invoice_components
                        .entry(roommate.clone())
                        .or_insert_with(|| vec![])
                        .push(InvoiceComponent {
                            label: String::from(label),
                            responsibility_proportion: share,
                            amount_due: shared_bill.amount_due(),
                            shared_amount: shared_bill.shared_amount(),
                        })
                }
                (shared_bill, split)
            })
            .collect::<Vec<_>>();
        self.split_bill_list(bill_list.iter().map(|(b, s)| (b, s)))
            .into_iter()
            .map(|(to, total)| {
                let components = invoice_components.remove(to).unwrap();
                let to = to.clone();
                Invoice {
                    to,
                    total,
                    components,
                }
            })
            .collect()
    }
}

fn estimate_shared_bills<'a, I>(
    label: &'a str,
    (current_bill, current_ti): (Bill, Option<f64>),
    history_with_ti: I,
    intervals: &ResponsibilityRecord,
) -> Result<(&'a str, SharedBill), Box<dyn Error>>
where
    I: IntoIterator<Item = (Bill, Option<f64>)>,
{
    let history = history_with_ti
        .into_iter()
        .map(|(bill, temperature_index)| {
            let occupancy = intervals.occupancy_over(bill.usage_period());
            (bill, occupancy, temperature_index)
        })
        .collect::<Vec<_>>();
    let borrowed_history = history.iter().map(|(b, ao, ti)| (b, *ao, *ti));
    let current_bill_notes = (
        intervals.occupancy_over(current_bill.usage_period()),
        current_ti,
    );
    Ok((
        label,
        SharedBill::from_estimate((current_bill, current_bill_notes), borrowed_history)?,
    ))
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
            "\t{} of the responsibility for the {} non-shared portion of the {} {} bill",
            self.responsibility_proportion,
            self.amount_due - self.shared_amount,
            self.amount_due,
            self.label
        )
    }
}
