pub(super) fn safe_div(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator.abs() < f64::EPSILON {
        None
    } else {
        Some(numerator / denominator)
    }
}

pub(super) fn growth(current: f64, previous: f64) -> Option<f64> {
    safe_div(current - previous, previous.abs())
}

pub(super) fn free_cash_flow(operating_cash_flow: f64, capital_expenditures: f64) -> f64 {
    operating_cash_flow - capital_expenditures.abs()
}

pub(super) fn display_value(value: f64, unit: &str) -> String {
    if unit == "ratio" {
        format!("{:.2}%", value * 100.0)
    } else if unit == "multiple" {
        format!("{value:.2}x")
    } else if unit == "USD" {
        format!("{value:.0}")
    } else {
        format!("{value:.4}")
    }
}

#[cfg(test)]
mod tests {
    use super::{free_cash_flow, growth, safe_div};

    #[test]
    fn growth_uses_absolute_previous_value() {
        assert_eq!(growth(120.0, 100.0), Some(0.2));
        assert_eq!(growth(-80.0, -100.0), Some(0.2));
        assert_eq!(growth(10.0, 0.0), None);
    }

    #[test]
    fn free_cash_flow_treats_capex_as_outflow() {
        assert_eq!(free_cash_flow(100.0, -30.0), 70.0);
        assert_eq!(free_cash_flow(100.0, 30.0), 70.0);
    }

    #[test]
    fn safe_div_rejects_zero_denominator() {
        assert_eq!(safe_div(1.0, 0.0), None);
        assert_eq!(safe_div(4.0, 2.0), Some(2.0));
    }
}
