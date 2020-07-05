use num::rational::Ratio;
use steel_cent::Money;

use super::interval::DateInterval;

#[derive(Debug)]
pub struct Bill {
    amount_due: Money,
    shared_cost: Money,
    period: DateInterval,
    usage_notes: UsageNotes,
}

impl Bill {
    pub fn new(amount_due: Money, shared_cost: Option<Money>, period: DateInterval) -> Self {
        let shared_cost = shared_cost.unwrap_or(amount_due);
        let usage_notes = UsageNotes {
            average_occupancy: None,
            temperature_index: None,
        };
        Bill {
            amount_due,
            shared_cost,
            period,
            usage_notes,
        }
    }

    pub fn amount_due(&self) -> &Money {
        &self.amount_due
    }

    pub fn shared_cost(&self) -> &Money {
        &self.shared_cost
    }

    pub fn set_shared_cost(&mut self, shared_cost: Money) {
        self.shared_cost = shared_cost;
    }

    pub fn period(&self) -> &DateInterval {
        &self.period
    }

    pub fn usage_notes(&self) -> &UsageNotes {
        &self.usage_notes
    }

    pub fn usage_notes_mut(&mut self) -> &mut UsageNotes {
        &mut self.usage_notes
    }
}

#[derive(Debug)]
pub struct UsageNotes {
    average_occupancy: Option<Ratio<u32>>,
    temperature_index: Option<f64>,
}

impl UsageNotes {
    pub fn average_occupancy(&self) -> &Option<Ratio<u32>> {
        &self.average_occupancy
    }

    pub fn update_average_occupancy(&mut self, average_occupancy: Ratio<u32>) {
        self.average_occupancy = Some(average_occupancy);
    }

    pub fn temperature_index(&self) -> &Option<f64> {
        &self.temperature_index
    }

    pub fn update_temperature_index(&mut self, temperature_index: f64) {
        self.temperature_index = Some(temperature_index);
    }
}
