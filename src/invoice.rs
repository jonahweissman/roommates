use itertools::Itertools;
use num::rational::Ratio;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use steel_cent::Money;

use super::bill::{Bill, SharedBill};
use super::interval::ResponsibilityInterval;
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
        responsibility_intervals: Vec<ResponsibilityInterval>,
    ) -> Vec<Invoice>
    where
        J: IntoIterator<Item = (&'a str, SharingData<I>)>,
        I: IntoIterator<Item = (Bill, Option<f64>)>,
    {
        let mut invoice_components: HashMap<Roommate, Vec<InvoiceComponent>> = HashMap::new();
        let bill_list = bills
            .into_iter()
            .map(|(label, sharing_data)| match sharing_data {
                SharingData::Variable(current_bill, history) => estimate_shared_bills(
                    label,
                    current_bill,
                    history,
                    responsibility_intervals.iter(),
                )
                .expect("estimating failed"),
                SharingData::Fixed(bill) => (
                    label,
                    SharedBill::from_fixed(Bill::new(
                        bill.amount_due(),
                        bill.period(),
                        Some(bill.amount_due()),
                    )),
                ),
            })
            .map(|(label, shared_bill)| {
                let split = self.individual_responsibilities(
                    responsibility_intervals.iter(),
                    shared_bill.period(),
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

fn estimate_shared_bills<'a, 'b, I, J>(
    label: &'a str,
    (current_bill, current_ti): (Bill, Option<f64>),
    history_with_ti: I,
    intervals: J,
) -> Result<(&'a str, SharedBill), Box<dyn Error>>
where
    I: IntoIterator<Item = (Bill, Option<f64>)>,
    J: IntoIterator<Item = &'b ResponsibilityInterval> + Clone,
{
    let history = history_with_ti
        .into_iter()
        .map(|(bill, temperature_index)| {
            let occupancy = bill.period().occupancy(intervals.clone().into_iter());
            (bill, occupancy, temperature_index)
        })
        .collect::<Vec<_>>();
    let borrowed_history = history.iter().map(|(b, ao, ti)| (b, *ao, *ti));
    let current_bill_notes = (
        current_bill
            .period()
            .occupancy(intervals.clone().into_iter()),
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
