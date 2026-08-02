use crate::domain::sheet::{CellError, CellValue};

/// Evaluates a function call with the given argument list.
pub fn evaluate_function(func_name: &str, args: &[CellValue]) -> CellValue {
    // Check for errors in arguments first
    for arg in args {
        if let CellValue::Error(e) = arg {
            return CellValue::Error(e.clone());
        }
    }

    match func_name {
        "POW" | "POWER" => func_pow(args),
        "SUM" => func_sum(args),
        "AVG" | "AVERAGE" => func_avg(args),
        "MAX" => func_max(args),
        "MIN" => func_min(args),
        "COUNT" => func_count(args),
        "SQRT" => func_sqrt(args),
        "MEDIAN" => func_median(args),
        _ => CellValue::Error(CellError::Syntax),
    }
}

fn func_pow(args: &[CellValue]) -> CellValue {
    if args.len() != 2 {
        return CellValue::Error(CellError::Syntax);
    }

    let base = match &args[0] {
        CellValue::Number(n) => *n,
        _ => return CellValue::Error(CellError::Value),
    };

    let exponent = match &args[1] {
        CellValue::Number(n) => *n,
        _ => return CellValue::Error(CellError::Value),
    };

    let result = base.powf(exponent);
    if result.is_finite() {
        CellValue::Number(result)
    } else {
        CellValue::Error(CellError::Value)
    }
}

fn func_sum(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Number(0.0);
    }

    let mut sum = 0.0;
    for arg in args {
        match arg {
            CellValue::Number(n) => sum += n,
            _ => return CellValue::Error(CellError::Value),
        }
    }
    CellValue::Number(sum)
}

fn func_avg(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Syntax);
    }

    let mut sum = 0.0;
    for arg in args {
        match arg {
            CellValue::Number(n) => sum += n,
            _ => return CellValue::Error(CellError::Value),
        }
    }
    CellValue::Number(sum / args.len() as f64)
}

fn func_max(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Syntax);
    }

    let mut max = f64::NEG_INFINITY;
    for arg in args {
        match arg {
            CellValue::Number(n) => {
                if *n > max {
                    max = *n;
                }
            }
            _ => return CellValue::Error(CellError::Value),
        }
    }
    CellValue::Number(max)
}

fn func_min(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Syntax);
    }

    let mut min = f64::INFINITY;
    for arg in args {
        match arg {
            CellValue::Number(n) => {
                if *n < min {
                    min = *n;
                }
            }
            _ => return CellValue::Error(CellError::Value),
        }
    }
    CellValue::Number(min)
}

fn func_count(args: &[CellValue]) -> CellValue {
    let mut count = 0;
    for arg in args {
        if let CellValue::Number(_) = arg {
            count += 1;
        }
    }
    CellValue::Number(count as f64)
}

fn func_sqrt(args: &[CellValue]) -> CellValue {
    if args.len() != 1 {
        return CellValue::Error(CellError::Syntax);
    }

    let val = match &args[0] {
        CellValue::Number(n) => *n,
        _ => return CellValue::Error(CellError::Value),
    };

    if val < 0.0 {
        return CellValue::Error(CellError::Value);
    }

    CellValue::Number(val.sqrt())
}

fn func_median(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Syntax);
    }

    let mut nums = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            CellValue::Number(n) => nums.push(*n),
            _ => return CellValue::Error(CellError::Value),
        }
    }

    if nums.is_empty() {
        return CellValue::Error(CellError::Syntax);
    }

    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = nums.len();
    let median = if len % 2 == 1 {
        nums[len / 2]
    } else {
        (nums[len / 2 - 1] + nums[len / 2]) / 2.0
    };

    CellValue::Number(median)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_pow() {
        assert_eq!(
            evaluate_function("POW", &[CellValue::Number(2.0), CellValue::Number(3.0)]),
            CellValue::Number(8.0)
        );
        assert_eq!(
            evaluate_function("POWER", &[CellValue::Number(3.0), CellValue::Number(2.0)]),
            CellValue::Number(9.0)
        );
    }

    #[test]
    fn test_func_sum() {
        assert_eq!(
            evaluate_function(
                "SUM",
                &[
                    CellValue::Number(1.0),
                    CellValue::Number(2.0),
                    CellValue::Number(3.0)
                ]
            ),
            CellValue::Number(6.0)
        );
        assert_eq!(evaluate_function("SUM", &[]), CellValue::Number(0.0));
    }

    #[test]
    fn test_func_avg() {
        assert_eq!(
            evaluate_function(
                "AVG",
                &[
                    CellValue::Number(10.0),
                    CellValue::Number(20.0),
                    CellValue::Number(30.0)
                ]
            ),
            CellValue::Number(20.0)
        );
        assert_eq!(
            evaluate_function("AVERAGE", &[CellValue::Number(4.0), CellValue::Number(8.0)]),
            CellValue::Number(6.0)
        );
    }

    #[test]
    fn test_func_max_min() {
        let args = vec![
            CellValue::Number(5.0),
            CellValue::Number(1.0),
            CellValue::Number(10.0),
        ];
        assert_eq!(evaluate_function("MAX", &args), CellValue::Number(10.0));
        assert_eq!(evaluate_function("MIN", &args), CellValue::Number(1.0));
    }

    #[test]
    fn test_func_count() {
        let args = vec![
            CellValue::Number(1.0),
            CellValue::Text("abc".to_string()),
            CellValue::Number(2.5),
        ];
        assert_eq!(evaluate_function("COUNT", &args), CellValue::Number(2.0));
    }

    #[test]
    fn test_func_sqrt() {
        assert_eq!(
            evaluate_function("SQRT", &[CellValue::Number(16.0)]),
            CellValue::Number(4.0)
        );
        assert_eq!(
            evaluate_function("SQRT", &[CellValue::Number(-4.0)]),
            CellValue::Error(CellError::Value)
        );
    }

    #[test]
    fn test_func_median() {
        // Odd count
        let args_odd = vec![
            CellValue::Number(5.0),
            CellValue::Number(1.0),
            CellValue::Number(3.0),
        ];
        assert_eq!(
            evaluate_function("MEDIAN", &args_odd),
            CellValue::Number(3.0)
        );

        // Even count
        let args_even = vec![
            CellValue::Number(1.0),
            CellValue::Number(2.0),
            CellValue::Number(3.0),
            CellValue::Number(4.0),
        ];
        assert_eq!(
            evaluate_function("MEDIAN", &args_even),
            CellValue::Number(2.5)
        );
    }
}
