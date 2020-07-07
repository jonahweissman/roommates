use linregress::{FormulaRegressionBuilder, RegressionDataBuilder, RegressionModel};
use num::ToPrimitive;
use std::collections::HashMap;
use steel_cent::Money;

use super::bill::Bill;

impl Bill {
    /// builds a linear model to predict amount due based on bill history,
    /// then applies the model to the current month, setting the average
    /// occupancy to zero
    ///
    /// Performs poorly if temperature index and average occupancy always vary
    /// together
    ///
    /// You will need several bills worth of data for this to work
    pub fn calculate_shared_cost<'a, I>(&'a mut self, bill_history: I)
    where
        I: IntoIterator<Item = &'a Bill>,
    {
        let (y, ao, ti) = extract_variables(bill_history.into_iter().collect());
        println!("{:?}", ao);
        let data: Vec<(&str, Vec<_>)> = vec![("Y", y), ("AO", ao), ("TI", ti)];
        let data = RegressionDataBuilder::new()
            .build_from(data)
            .expect("Error while building regression data");
        let formula = "Y ~ AO + TI";
        let model = FormulaRegressionBuilder::new()
            .data(&data)
            .formula(formula)
            .fit()
            .expect("something went wrong with the regression fitting");
        self.assess_model(&model);
        let intercept_value = model.parameters.intercept_value;
        let parameters: HashMap<_, _> = model.parameters.pairs().into_iter().collect();
        let shared_cost = intercept_value
            + *parameters.get("TI").unwrap()
                * self.usage_notes().temperature_index().unwrap_or(0.0);
        let currency = self.amount_due().currency;
        self.set_shared_cost(Money::of_minor(currency, shared_cost as i64));
        assert!(model.rsquared >= 0.80, format!("shared cost model fits poorly ({})", &model.rsquared));
    }

    /// |actual - predicted| / actual
    ///
    /// closer to zero is better
    fn assess_model(&self, model: &RegressionModel) -> f64 {
        let data = vec![
            (
                "AO",
                vec![self
                    .usage_notes()
                    .average_occupancy()
                    .unwrap()
                    .to_f64()
                    .unwrap()],
            ),
            (
                "TI",
                vec![self.usage_notes().temperature_index().unwrap_or(0.0)],
            ),
        ];
        let predicted = model.predict(data).unwrap()[0];
        let actual = self.amount_due().minor_amount() as f64;
        let error = (predicted - actual).abs() / actual;
        assert!(
            error <= 0.2,
            format!(
                "model poorly predicts most recent bill ({}) as {}",
                actual,
                predicted,
            )
        );
        error
    }
}

fn extract_variables(bill_history: Vec<&Bill>) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (cost_ao, ti): (Vec<_>, Vec<_>) = bill_history
        .iter()
        .map(|bill| match bill.usage_notes().average_occupancy() {
            None => None,
            Some(average_occupancy) => Some((
                (
                    bill.amount_due().minor_amount() as f64,
                    average_occupancy.to_f64().unwrap(),
                ),
                bill.usage_notes().temperature_index().unwrap_or(0.0),
            )),
        })
        .filter_map(|x| x)
        .unzip();
    let (cost, ao) = cost_ao.into_iter().unzip();
    (cost, ao, ti)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval::DateInterval;
    use num::rational::Ratio;
    use steel_cent::currency::USD;

    fn build_bills(history: Vec<(i64, u32)>, current: (i64, u32)) -> (Vec<Bill>, Bill) {
        let bills = history
            .into_iter()
            .map(|(m, ao)| {
                let mut b = Bill::new(
                    Money::of_minor(USD, m),
                    DateInterval::from_strs("01/01/20", "02/01/20"),
                    None,
                );
                b.usage_notes_mut()
                    .update_average_occupancy(Ratio::from_integer(ao));
                b
            })
            .collect::<Vec<Bill>>();
        let mut current_bill = Bill::new(
            Money::of_minor(USD, current.0),
            DateInterval::from_strs("04/01/20", "05/01/20"),
            None,
        );
        current_bill
            .usage_notes_mut()
            .update_average_occupancy(Ratio::from_integer(current.1));
        (bills, current_bill)
    }

    fn build_bills_ti(
        history: Vec<(i64, u32)>,
        current: (i64, u32, f64),
        ti: Vec<f64>,
    ) -> (Vec<Bill>, Bill) {
        assert_eq!(history.len(), ti.len());
        assert!(history.len() > 1);
        let (mut bills, mut current_bill) = build_bills(history, (current.0, current.1));
        current_bill
            .usage_notes_mut()
            .update_temperature_index(current.2);
        for (bill, ti) in bills.iter_mut().zip(ti.into_iter()) {
            bill.usage_notes_mut().update_temperature_index(ti);
        }
        (bills, current_bill)
    }

    #[test]
    fn get_variables_from_simple_bill_history() {
        let (bills, _) = build_bills(vec![(20_00, 1), (30_00, 2), (40_00, 3)], (50_00, 0));
        let (y, x1, x2) = extract_variables(bills.iter().collect());
        assert_eq!(y, vec![2000.0, 3000.0, 4000.0]);
        assert_eq!(x1, vec![1.0, 2.0, 3.0]);
        assert_eq!(x2, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn simple_bill_history_shared_cost() {
        let (bills, mut current): (Vec<Bill>, Bill) = build_bills(
            vec![(10_00, 0), (20_00, 1), (30_00, 2), (40_00, 3)],
            (50_00, 4),
        );
        current.calculate_shared_cost(bills.iter().collect::<Vec<_>>());
        assert!((current.shared_cost().unwrap() - Money::of_minor(USD, 10_00)) <= Money::of_minor(USD, 1));
    }

    #[test]
    fn bill_history_with_temperature() {
        let (bills, mut current) = build_bills_ti(
            vec![(24_00, 1), (34_00, 2), (42_00, 3), (43_00, 3)],
            (41_50, 3, 1.0),
            vec![4.0, 4.0, 2.0, 3.0],
        );
        current.calculate_shared_cost(bills.iter().collect::<Vec<_>>());
        assert_eq!(current.shared_cost().unwrap(), Money::of_minor(USD, 11_00));
    }
}
