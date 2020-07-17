use linregress::{FormulaRegressionBuilder, RegressionDataBuilder, RegressionModel};
use std::collections::HashMap;
use std::error::Error;
use steel_cent::Money;

use super::bill::{Bill, SharedBill};

impl SharedBill {
    /// builds a linear model to predict amount due based on bill history,
    /// then applies the model to the current month, setting the
    /// occupancy to zero
    ///
    /// Performs poorly if temperature index and occupancy always vary
    /// together
    ///
    /// You will need several bills worth of data for this to work
    pub fn from_estimate<'a, I>(
        bill_with_notes: (Bill, (u32, Option<f64>)),
        bill_history: I,
    ) -> Result<Self, Box<dyn Error>>
    where
        I: IntoIterator<Item = (&'a Bill, u32, Option<f64>)>,
    {
        let (y, oc, ti) = extract_variables(bill_history);
        let data: Vec<(&str, Vec<_>)> = vec![("Y", y), ("Oc", oc), ("TI", ti)];
        let data = RegressionDataBuilder::new().build_from(data)?;
        let formula = "Y ~ Oc + TI";
        let model = FormulaRegressionBuilder::new()
            .data(&data)
            .formula(formula)
            .fit()?;
        let (bill, notes) = bill_with_notes;
        bill.assess_model(&model, notes);
        assert!(
            model.rsquared >= 0.80,
            format!("shared cost model fits poorly ({})", &model.rsquared)
        );
        let intercept_value = model.parameters.intercept_value;
        let parameters: HashMap<_, _> = model.parameters.pairs().into_iter().collect();
        let shared_cost = 0.0f64.max((bill.amount_due().minor_amount() as f64).min(
            intercept_value
                + *parameters.get("TI").unwrap() * notes.1.unwrap_or(0.0)
                + bill.fixed_cost().minor_amount() as f64,
        ));
        let currency = bill.amount_due().currency;
        Ok(SharedBill::new(
            bill,
            Money::of_minor(currency, shared_cost as i64),
        ))
    }

    pub fn from_fixed(bill: Bill) -> Self {
        let shared_amount = bill.fixed_cost();
        SharedBill::new(bill, shared_amount)
    }
}
impl Bill {
    /// |actual - predicted| / actual
    ///
    /// closer to zero is better
    fn assess_model(&self, model: &RegressionModel, notes: (u32, Option<f64>)) -> f64 {
        let (oc, ti) = notes;
        let data = vec![("Oc", vec![oc as f64]), ("TI", vec![ti.unwrap_or(0.0)])];
        let predicted = model.predict(data).unwrap()[0];
        let actual = self.amount_due().minor_amount() as f64;
        let error = (predicted - actual).abs() / actual;
        assert!(
            error <= 0.2,
            format!(
                "model poorly predicts most recent bill ({}) as {}",
                actual, predicted,
            )
        );
        error
    }
}

fn extract_variables<'a, I>(bill_history: I) -> (Vec<f64>, Vec<f64>, Vec<f64>)
where
    I: IntoIterator<Item = (&'a Bill, u32, Option<f64>)>,
{
    let (cost_oc, ti): (Vec<_>, Vec<_>) = bill_history
        .into_iter()
        .map(|(bill, occupancy, temperature_index)| {
            (
                (
                    (bill.amount_due().minor_amount() - bill.fixed_cost().minor_amount()) as f64,
                    occupancy as f64,
                ),
                temperature_index.unwrap_or(0.0),
            )
        })
        .unzip();
    let (cost, oc) = cost_oc.into_iter().unzip();
    (cost, oc, ti)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval::DateInterval;
    use steel_cent::currency::USD;

    fn build_bills(
        history: Vec<(i64, u32)>,
        current: (i64, u32),
    ) -> (Vec<(Bill, u32, Option<f64>)>, Bill, (u32, Option<f64>)) {
        let bills = history
            .into_iter()
            .map(|(m, oc)| {
                (
                    Bill::new(
                        Money::of_minor(USD, m),
                        DateInterval::from_strs("01/01/2020", "02/01/2020").unwrap(),
                        None,
                    ),
                    oc,
                    None,
                )
            })
            .collect::<Vec<_>>();
        let current_bill = Bill::new(
            Money::of_minor(USD, current.0),
            DateInterval::from_strs("04/01/2020", "05/01/2020").unwrap(),
            None,
        );
        let notes = (current.1, None);
        (bills, current_bill, notes)
    }

    fn build_bills_ti(
        history: Vec<(i64, u32)>,
        current: (i64, u32, f64),
        ti: Vec<f64>,
    ) -> (Vec<(Bill, u32, Option<f64>)>, Bill, (u32, Option<f64>)) {
        assert_eq!(history.len(), ti.len());
        assert!(history.len() > 1);
        let (bills, current_bill, notes) = build_bills(history, (current.0, current.1));
        let (oc, _) = notes;
        let notes = (oc, Some(current.2));
        let history = bills
            .into_iter()
            .zip(ti.into_iter())
            .map(|((bill, oc, _), ti)| (bill, oc, Some(ti)))
            .collect::<Vec<_>>();
        (history, current_bill, notes)
    }

    #[test]
    fn get_variables_from_simple_bill_history() {
        let (bills, _, _) = build_bills(vec![(20_00, 1), (30_00, 2), (40_00, 3)], (50_00, 0));
        let (y, x1, x2) = extract_variables(
            bills
                .iter()
                .map(|(b, oc, ti)| (b, *oc, *ti))
                .collect::<Vec<_>>(),
        );
        assert_eq!(y, vec![2000.0, 3000.0, 4000.0]);
        assert_eq!(x1, vec![1.0, 2.0, 3.0]);
        assert_eq!(x2, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn simple_bill_history_shared_cost() {
        let (bills, current, notes) = build_bills(
            vec![(10_00, 0), (20_00, 1), (30_00, 2), (40_00, 3)],
            (50_00, 4),
        );
        let current = SharedBill::from_estimate(
            (current, notes),
            bills
                .iter()
                .map(|(b, oc, ti)| (b, *oc, *ti))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert!((current.shared_amount() - Money::of_minor(USD, 10_00)) <= Money::of_minor(USD, 1));
    }

    #[test]
    fn bill_history_with_temperature() {
        let (bills, current, notes) = build_bills_ti(
            vec![(24_00, 1), (34_00, 2), (42_00, 3), (43_00, 3)],
            (41_50, 3, 1.0),
            vec![4.0, 4.0, 2.0, 3.0],
        );
        let current = SharedBill::from_estimate(
            (current, notes),
            bills
                .iter()
                .map(|(b, oc, ti)| (b, *oc, *ti))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert_eq!(current.shared_amount(), Money::of_minor(USD, 11_00));
    }

    #[test]
    fn bill_with_fixed_cost() {
        let bill_history = vec![
            (
                Bill::new(
                    Money::of_minor(USD, 110_00),
                    DateInterval::from_strs("01/01/2001", "02/02/2002").unwrap(),
                    Some(Money::of_minor(USD, 100_00)),
                ),
                0,
                None,
            ),
            (
                Bill::new(
                    Money::of_minor(USD, 120_00),
                    DateInterval::from_strs("01/01/2001", "02/02/2002").unwrap(),
                    Some(Money::of_minor(USD, 100_00)),
                ),
                1,
                None,
            ),
            (
                Bill::new(
                    Money::of_minor(USD, 30_00),
                    DateInterval::from_strs("01/01/2001", "02/02/2002").unwrap(),
                    None,
                ),
                2,
                None,
            ),
            (
                Bill::new(
                    Money::of_minor(USD, 40_00),
                    DateInterval::from_strs("01/01/2001", "02/02/2002").unwrap(),
                    None,
                ),
                3,
                None,
            ),
        ];
        let bill = Bill::new(
            Money::of_minor(USD, 50_00),
            DateInterval::from_strs("01/01/2001", "02/02/2002").unwrap(),
            Some(Money::of_minor(USD, 10_00)),
        );
        let notes = (4, None);
        let current = SharedBill::from_estimate(
            (bill, notes),
            bill_history
                .iter()
                .map(|(b, oc, ti)| (b, *oc, *ti))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert!((current.shared_amount() - Money::of_minor(USD, 20_00)) <= Money::of_minor(USD, 1));
    }
}
