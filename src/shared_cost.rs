use linregress::{
    Error as LinregressError, FormulaRegressionBuilder, RegressionDataBuilder, RegressionModel,
};
use std::collections::HashMap;
use std::iter;
use steel_cent::Money;
use thiserror::Error;

use super::bill::{Bill, SharedBill};
use super::interval::ResponsibilityRecord;

#[derive(Debug, Error, PartialEq)]
pub enum EstimationError {
    #[error("Something is wrong with the bill history: {0}")]
    InvalidModelData(String),

    #[error(
        "A good estimator could not be created from the given bill history \
        (rsquared == {0})"
    )]
    ModelFitsDataPoorly(f64),

    #[error(
        "The model has a high mean absolute percentage error of {0} predicting the given bill"
    )]
    ModelPredictsPoorly(f64),
}

impl SharedBill {
    /// Creates a `SharedBill` by estimating the shared cost
    ///
    /// Builds a linear model to predict amount due based on bill history,
    /// then applies the model to the current month, setting the
    /// occupancy to zero
    ///
    /// Performs poorly if temperature index and occupancy always vary
    /// together
    /// 
    /// make sure to use the `variable_cost` for the DependentaVariable
    /// not the amount due
    ///
    /// You will need several bills worth of data for this to work
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill, DependentVariable, Occupancy}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let current_bill = Bill::new(
    ///     Money::of_minor(USD, 40_27),
    ///     DateInterval::from_strs("01/01/01", "02/02/02").unwrap(),
    /// );
    /// let to_money = |DependentVariable(x)| Money::of_minor(USD, x as i64);
    /// let current_data = (Occupancy(30), DependentVariable(40_27.0));
    /// let bill_history = vec![
    ///     (Occupancy(10), DependentVariable(20_00.0)),
    ///     (Occupancy(15), DependentVariable(25_00.0)),
    ///     (Occupancy(20), DependentVariable(30_00.0)),
    /// ];
    /// let shared = SharedBill::from_estimate(
    ///     (current_bill, current_data),
    ///     bill_history,
    ///     to_money,
    /// ).unwrap();
    /// assert_eq!(shared.shared_amount(), Money::of_minor(USD, 10_00));
    /// ```
    pub fn from_estimate<I, X, F>(
        (bill, (bill_x, bill_y)): (Bill, (X, DependentVariable)),
        bill_history: I,
        to_money: F,
    ) -> Result<Self, EstimationError>
    where
        I: IntoIterator<Item = (X, DependentVariable)>,
        X: IndependentVariable,
        F: Fn(DependentVariable) -> Money + Copy,
    {
        let model = SharedCostEstimator::new(bill_history, to_money)?;
        if model.rsquared() < 0.70 {
            return Err(EstimationError::ModelFitsDataPoorly(model.rsquared()));
        };
        let error = model.mape((&bill_x, bill_y));
        if error > 0.2 {
            return Err(EstimationError::ModelPredictsPoorly(error));
        };
        // add explicit fixed cost back in and restrict to range [0, amount_due]
        let shared_cost = (model.predict_empty(&bill_x).minor_amount()
            + bill.fixed_cost().minor_amount())
        .min(bill.amount_due().minor_amount())
        .max(0);
        let currency = bill.amount_due().currency;
        Ok(SharedBill::new(bill, Money::of_minor(currency, shared_cost as i64)).unwrap())
    }

    /// Creates a new `SharedBill`, using the fixed cost as the shared cost
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, 40_00),
    /// ).unwrap();
    /// let shared_water_bill = SharedBill::from_fixed(water_bill);
    /// assert_eq!(
    ///     shared_water_bill.shared_amount(),
    ///     Money::of_minor(USD, 40_00),
    /// );
    /// ```
    pub fn from_fixed(bill: Bill) -> Self {
        let shared_amount = bill.fixed_cost();
        // the fixed cost of a Bill is always valid
        SharedBill::new(bill, shared_amount).unwrap()
    }

    /// Creates a new `Bill` with the full amount shared
    ///
    /// # Examples
    /// ```
    /// use roommates::{DateInterval, sharing::{Bill, SharedBill}};
    /// use steel_cent::{Money, currency::USD};
    ///
    /// let water_bill = Bill::new_with_fixed_cost(
    ///     Money::of_minor(USD, 83_22),
    ///     DateInterval::new((2020, 4, 15), (2020, 5, 15)).unwrap(),
    ///     Money::of_minor(USD, 40_00),
    /// ).unwrap();
    /// let shared_water_bill = SharedBill::from_fully_fixed(water_bill);
    /// assert_eq!(
    ///     shared_water_bill.shared_amount(),
    ///     Money::of_minor(USD, 83_22),
    /// );
    /// ```
    pub fn from_fully_fixed(bill: Bill) -> SharedBill {
        let amount_due = bill.amount_due();
        SharedBill::new(bill, amount_due).unwrap()
    }
}

/// Predicts the cost of a bill with an empty house
///
/// to_money is applied to the output of the internal `RegressionModel` to
/// convert it to a `Money` value for use as a `shared_amount`.
pub struct SharedCostEstimator<F: Fn(DependentVariable) -> Money> {
    model: RegressionModel,
    to_money: F,
}

impl<F: Fn(DependentVariable) -> Money> SharedCostEstimator<F> {
    /// Returns a new `SharedCostEstimator` fitted on the given data
    fn new<'a, X, I>(bill_history: I, to_money: F) -> Result<Self, EstimationError>
    where
        I: IntoIterator<Item = (X, DependentVariable)>,
        X: IndependentVariable,
    {
        let data = RegressionDataBuilder::new()
            .build_from(extract_variables(bill_history))
            .map_err(|source| match source {
                LinregressError::RegressionDataError(message) => {
                    EstimationError::InvalidModelData(message)
                }
                _ => panic!("Unrecognized error from linregress: {:?}", source),
            })?;
        let formula = String::from("Y ~ ") + X::formula();
        let model = FormulaRegressionBuilder::new()
            .data(&data)
            .formula(formula)
            .fit()
            .map_err(|source| match source {
                LinregressError::NoData => panic!("Missing fitting data (should never happen)"),
                LinregressError::NoFormula => panic!("Missing formula (should never happen"),
                LinregressError::InvalidFormula => {
                    panic!("Implementors of IndendentVariable must provide valid formulas")
                }
                LinregressError::ColumnNotInData(col) => panic!(
                    "Implentors of IndependentVariable must ensure that formula is \
                           consistent with other methods; column not in data: {}",
                    col
                ),
                LinregressError::RegressorRegressandDimensionMismatch(_) =>
                    panic!("RegressorRegressandDimensionMismatch"),
                _ => panic!("Unrecognized error from linregress: {:?}", source),
            })?;
        Ok(SharedCostEstimator {
            model,
            to_money,
        })
    }

    /// Returns the predicted cost of a bill given some `IndependentVariable`
    fn predict<X>(&self, x: &X) -> Money
    where
        X: IndependentVariable,
    {
        let raw_output = self
            .model
            .predict(
                x.to_linregress_data_fmt()
                    .into_iter()
                    .map(|(s, val)| (s, vec![val])),
            )
            .map_err(|source| match source {
                LinregressError::NoData => panic!("Implementors of `IndependentVariable` must return data in their `to_linregress_data_fmt` method"),
                LinregressError::InconsistentRegressionModel => panic!("The estimation model is broken (should never happen)"),
                LinregressError::ColumnNotInData(col) => panic!("Input columns don't line up, extra column: {} (should never happen)", col),
                LinregressError::ModelColumnNotInData(col) => panic!("Input columns don't line up, extra column: {} (should never happen)", col),
                _ => panic!("Unrecognized error from linregress: {:?}", source),
            }).unwrap()[0];
        (self.to_money)(DependentVariable(raw_output))
    }

    /// Returns the predicted cost of a bill if the housing unit had been empty
    ///
    /// takes in an `IndependentVariable` because it may contain information other
    /// than occupancy
    fn predict_empty<X>(&self, x: &X) -> Money
    where
        X: IndependentVariable,
    {
        self.predict::<X>(&x.empty())
    }

    fn rsquared(&self) -> f64 {
        self.model.rsquared
    }

    /// returns the mean absolute percentage error on an (x, y) pair
    ///
    /// |actual - predicted| / actual
    /// closer to zero is better
    fn mape<X>(&self, (x, y): (&X, DependentVariable)) -> f64
    where
        X: IndependentVariable,
    {
        let predicted = self.predict(x).minor_amount() as f64;
        let actual = (self.to_money)(y).minor_amount() as f64;
        (actual - predicted).abs() / actual
    }
}

/// Returns the given data in the format expected by `linregress`
fn extract_variables<I, X>(bill_history: I) -> HashMap<&'static str, Vec<f64>>
where
    I: IntoIterator<Item = (X, DependentVariable)>,
    X: IndependentVariable,
{
    let mut data = HashMap::new();
    for (x, DependentVariable(y)) in bill_history {
        for (label, val) in x
            .to_linregress_data_fmt()
            .into_iter()
            .chain(iter::once(("Y", y)))
        {
            data.entry(label).or_insert_with(Vec::new).push(val);
        }
    }
    data
}

/// represents any usage related
/// measurement. This can be the minor amount of the bill cost or
/// kWh or anything like that.  
pub struct DependentVariable(pub f64);

/// An input for building a `SharedCostEstimator`
///
/// each instance represents a single data point
///
/// can contain any number of variables
///
/// See example OccupancyAndTemperature
pub trait IndependentVariable {
    /// Returns vector of (name, value) pairs for the current data point
    fn to_linregress_data_fmt(&self) -> Vec<(&'static str, f64)>;

    /// returns str representation of the sum of variables
    ///
    /// "X1 + X2 + Xn"
    fn formula() -> &'static str;

    /// sets occupancy data to zero
    fn empty(&self) -> Self;
}

pub struct Occupancy(pub u32);
pub struct OccupancyAndTemperature(pub u32, pub f64);

impl IndependentVariable for Occupancy {
    fn to_linregress_data_fmt(&self) -> Vec<(&'static str, f64)> {
        vec![("Occupancy", self.0 as f64)]
    }

    fn formula() -> &'static str {
        "Occupancy"
    }

    fn empty(&self) -> Self {
        Occupancy(0)
    }
}

impl IndependentVariable for OccupancyAndTemperature {
    fn to_linregress_data_fmt(&self) -> Vec<(&'static str, f64)> {
        vec![("Occupancy", self.0 as f64), ("TemperatureIndex", self.1)]
    }

    fn formula() -> &'static str {
        "Occupancy + TemperatureIndex"
    }

    fn empty(&self) -> Self {
        let OccupancyAndTemperature(_, ti) = &self;
        OccupancyAndTemperature(0, *ti)
    }
}

/// Estimates a SharedBill from a `Bill` history using `Occupancy`
pub fn convert_to_shared<I>(
    (bill, history): (Bill, I),
    intervals: &ResponsibilityRecord,
) -> Result<SharedBill, EstimationError>
where
    I: IntoIterator<Item = Bill>,
{
    let period = bill.usage_period();
    let amount_due = bill.variable_cost();
    let to_money = |DependentVariable(x)| Money::of_minor(amount_due.currency, x as i64);
    Ok(SharedBill::from_estimate(
        (
            bill,
            (
                Occupancy(intervals.occupancy_over(period)),
                DependentVariable(amount_due.minor_amount() as f64),
            ),
        ),
        history.into_iter().map(|b| {
            (
                Occupancy(intervals.occupancy_over(b.usage_period())),
                DependentVariable(b.amount_due().minor_amount() as f64),
            )
        }),
        to_money,
    )?)
}

pub fn convert_to_shared_ti<I>(
    ((bill, current_ti), history): ((Bill, f64), I),
    intervals: &ResponsibilityRecord,
) -> Result<SharedBill, EstimationError>
where
    I: IntoIterator<Item = (Bill, f64)>,
{
    let period = bill.usage_period();
    let amount_due = bill.variable_cost();
    let to_money = |DependentVariable(x)| Money::of_minor(amount_due.currency, x as i64);
    Ok(SharedBill::from_estimate(
        (
            bill,
            (
                OccupancyAndTemperature(intervals.occupancy_over(period), current_ti),
                DependentVariable(amount_due.minor_amount() as f64),
            ),
        ),
        history.into_iter().map(|(b, ti)| {
            (
                OccupancyAndTemperature(intervals.occupancy_over(b.usage_period()), ti),
                DependentVariable(b.amount_due().minor_amount() as f64),
            )
        }),
        to_money,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval::DateInterval;
    use steel_cent::currency::USD;

    fn build_bills(
        history: Vec<(i64, u32)>,
        (current_amount, current_oc): (i64, u32),
    ) -> (
        Vec<(Occupancy, DependentVariable)>,
        Bill,
        (Occupancy, DependentVariable),
    ) {
        let bills = history
            .into_iter()
            .map(|(m, oc)| (Occupancy(oc), DependentVariable(m as f64)))
            .collect::<Vec<_>>();
        let bill = Bill::new(
            Money::of_minor(USD, current_amount),
            DateInterval::from_strs("01/01/01", "02/02/02").unwrap(),
        );
        let notes = (
            Occupancy(current_oc),
            DependentVariable(current_amount as f64),
        );
        (bills, bill, notes)
    }

    fn build_bills_ti(
        history: Vec<(i64, u32, f64)>,
        (current_amount, current_oc, current_ti): (i64, u32, f64),
    ) -> (
        Vec<(OccupancyAndTemperature, DependentVariable)>,
        Bill,
        (OccupancyAndTemperature, DependentVariable),
    ) {
        assert!(history.len() > 1);
        let bills = history
            .into_iter()
            .map(|(m, oc, ti)| (OccupancyAndTemperature(oc, ti), DependentVariable(m as f64)))
            .collect::<Vec<_>>();
        let bill = Bill::new(
            Money::of_minor(USD, current_amount),
            DateInterval::from_strs("01/01/01", "02/02/02").unwrap(),
        );
        let notes = (
            OccupancyAndTemperature(current_oc, current_ti),
            DependentVariable(current_amount as f64),
        );
        (bills, bill, notes)
    }

    #[test]
    fn get_variables_from_simple_bill_history() {
        let (bills, _, _) = build_bills(vec![(20_00, 1), (30_00, 2), (40_00, 3)], (50_00, 0));
        let variables = extract_variables(bills);
        assert_eq!(variables.get("Y").unwrap(), &vec![2000.0, 3000.0, 4000.0]);
        assert_eq!(variables.get("Occupancy").unwrap(), &vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn simple_bill_history_shared_cost() {
        let (bills, current, notes) = build_bills(
            vec![(10_00, 0), (20_00, 1), (30_00, 2), (40_00, 3)],
            (50_00, 4),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x as i64)
        })
        .unwrap();
        assert!((current.shared_amount() - Money::of_minor(USD, 10_00)) <= Money::of_minor(USD, 1));
    }

    #[test]
    fn estimate_with_bad_prediction() {
        let (bills, current, notes) = build_bills(
            vec![(10_00, 0), (20_00, 1), (30_00, 2), (40_00, 3)],
            (20_00, 4),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x as i64)
        });
        assert!(matches!(current, Err(EstimationError::ModelPredictsPoorly(_))));
    }

    #[test]
    fn estimate_with_bad_fit() {
        let (bills, current, notes) = build_bills(
            vec![(50_00, 0), (20_00, 1), (30_00, 2), (40_00, 3)],
            (50_00, 4),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x as i64)
        });
        assert!(matches!(current, Err(EstimationError::ModelFitsDataPoorly(_))));
    }

    #[test]
    fn estimate_with_bad_data() {
        let (bills, current, notes) = build_bills(
            vec![(20_00, 1), (20_00, 1), (20_00, 1), (20_00, 1)],
            (50_00, 4),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x as i64)
        });
        assert!(matches!(current, Err(EstimationError::InvalidModelData(_))));
    }

    #[test]
    fn bill_history_with_temperature() {
        let (bills, current, notes) = build_bills_ti(
            vec![(24_00, 1, 4.0), (34_00, 2, 4.0), (42_00, 3, 2.0), (43_00, 3, 3.0)],
            (41_50, 3, 1.0),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x as i64)
        })
        .unwrap();
        assert_eq!(current.shared_amount(), Money::of_minor(USD, 11_00));
    }

    #[test]
    fn scaled_cost_function() {
        let (bills, current, notes) = build_bills_ti(
            vec![(24_00, 1, 4.0), (34_00, 2, 4.0), (42_00, 3, 2.0), (43_00, 3, 3.0)],
            (41_50, 3, 1.0),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, 2 * x as i64)
        })
        .unwrap();
        assert_eq!(current.shared_amount(), Money::of_minor(USD, 2 * 11_00));
    }

    #[test]
    fn nonlinear_cost_function() {
        let (bills, current, notes) = build_bills_ti(
            vec![(24_00, 1, 4.0), (34_00, 2, 4.0), (42_00, 3, 2.0), (43_00, 3, 3.0)],
            (41_50, 3, 1.0),
        );
        let current = SharedBill::from_estimate((current, notes), bills, |DependentVariable(x)| {
            Money::of_minor(USD, x.sqrt() as i64)
        })
        .unwrap();
        assert_eq!(
            current.shared_amount(),
            Money::of_minor(USD, (11_00 as f64).sqrt() as _)
        );
    }
}
