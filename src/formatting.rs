pub fn format_g(value: f64, precision: usize) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if !value.is_finite() {
        return format!("{}", value);
    }

    let scientific = format!("{:.*e}", precision - 1, value);
    let (mantissa, exponent_text) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent_text.parse().unwrap();
    let p = precision as i32;

    if exponent < -4 || exponent >= p {
        let mantissa = strip_trailing_zeros(mantissa);
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{}{:02}", mantissa, sign, exponent.abs())
    } else {
        let decimals = (p - 1 - exponent).max(0) as usize;
        let fixed = format!("{:.*}", decimals, value);
        strip_trailing_zeros(&fixed)
    }
}

pub fn to_string_double(value: f64) -> String {
    format!("{:.6}", value)
}

fn strip_trailing_zeros(text: &str) -> String {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text.to_string()
    }
}
