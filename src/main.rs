use core::f64;
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    usize,
};

const DEFAULT_PRECISION: usize = 5;

struct Measure {
    value: f64,
    uncertainty: Uncertainty,
}

impl Measure {
    fn scalar(value: f64) -> Measure {
        Measure {
            value,
            uncertainty: Uncertainty::null(),
        }
    }

    fn with_rel_err(self, err: f64) -> Measure {
        Measure {
            value: self.value,
            uncertainty: Uncertainty::symmetric_rel(err),
        }
    }

    fn with_abs_err(self, err: f64) -> Measure {
        Measure {
            value: self.value,
            uncertainty: Uncertainty::symmetric_abs(err),
        }
    }

    fn with_precision(&self, precision: usize) -> Self {
        Self {
            value: self.value,
            uncertainty: self.uncertainty.with_precision(precision),
        }
    }

    fn resolve_high_low_limits(&self) -> (f64, f64) {
        let unc = self.uncertainty.to_absolute(self.value);
        (self.value + unc.high, self.value - unc.low)
    }
}
#[derive(Copy, Clone, Debug)]
enum UncertaintyVariant {
    Absolute,
    Relative,
}

#[derive(Copy, Clone, Debug)]
struct Uncertainty {
    low: f64,
    high: f64,
    precision: usize,
    variant: UncertaintyVariant,
}

#[allow(dead_code)]
impl Uncertainty {
    fn null() -> Self {
        Self {
            low: 0.,
            high: 0.,
            precision: DEFAULT_PRECISION,
            variant: UncertaintyVariant::Absolute,
        }
    }

    fn symmetric_abs(value: f64) -> Self {
        Self {
            low: value.abs(),
            high: value.abs(),
            variant: UncertaintyVariant::Absolute,
            ..Self::null()
        }
    }
    fn symmetric_rel(value: f64) -> Self {
        Self {
            low: value.abs(),
            high: value.abs(),
            variant: UncertaintyVariant::Relative,
            ..Self::null()
        }
    }

    fn non_symmetric(low: f64, high: f64) -> Self {
        Self {
            low,
            high,
            ..Self::null()
        }
    }

    fn with_precision(self, precision: usize) -> Self {
        Self { precision, ..self }
    }

    fn to_absolute(self, value: f64) -> Uncertainty {
        if let UncertaintyVariant::Relative = self.variant {
            Self {
                low: (self.low / 100.) * value.abs(),
                high: (self.high / 100.) * value.abs(),
                variant: UncertaintyVariant::Absolute,
                ..self
            }
        } else {
            self
        }
    }
    fn to_relative(self, value: f64) -> Uncertainty {
        if let UncertaintyVariant::Absolute = self.variant {
            Self {
                low: (self.low - value.abs()).abs() / value.abs() * 100.,
                high: (self.high - value.abs()).abs() / value.abs() * 100.,
                variant: UncertaintyVariant::Relative,
                ..self
            }
        } else {
            self
        }
    }
}

fn main() {
    let a = Measure::scalar(1.23).with_precision(1) + Measure::scalar(2.0).with_rel_err(50.);
    let b = Measure::scalar(1.25) - Measure::scalar(2.0).with_rel_err(50.).with_precision(3);
    let c = Measure::scalar(1.28).with_precision(1) * Measure::scalar(2.0).with_abs_err(0.1);
    let d = Measure::scalar(12.) / Measure::scalar(2.91).with_abs_err(0.1).with_precision(2);

    println!("a \t= {}", &a);
    println!("b \t= {}", &b);
    println!("c \t= {}", &c);
    println!("d \t= {}", &d);
    println!("a + d \t= {}", a + d);
    println!("b * c \t= {}", b * c);
}

impl Display for Measure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (high, low) = self.resolve_high_low_limits();
        write!(
            f,
            "{:>pad$.precision$} {:>pad$.precision$} \n\t:= ({:>pad$.precision$}, {:>pad$.precision$})",
            self.value,
            self.uncertainty,
            low,
            high,
            precision = self.uncertainty.precision,
            pad = 0
        )
    }
}

impl Display for Uncertainty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.low != self.high {
            write!(
                f,
                "+{:.precision$}{symbol} / -{:.precision$}{symbol}",
                self.high,
                self.low,
                precision = self.precision,
                symbol = match self.variant {
                    UncertaintyVariant::Absolute => "",
                    UncertaintyVariant::Relative => "%",
                }
            )
        } else {
            write!(
                f,
                "±{:.precision$}{symbol}",
                self.high,
                precision = self.precision,
                symbol = match self.variant {
                    UncertaintyVariant::Absolute => "",
                    UncertaintyVariant::Relative => "%",
                }
            )
        }
    }
}

impl Default for Uncertainty {
    fn default() -> Self {
        Self::null()
    }
}

impl Add for Measure {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            uncertainty: self.uncertainty.to_absolute(self.value)
                + rhs.uncertainty.to_absolute(rhs.value),
        }
    }
}

impl Sub for Measure {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value - rhs.value,
            uncertainty: self.uncertainty.to_absolute(self.value)
                + rhs.uncertainty.to_absolute(rhs.value),
        }
    }
}

impl Add for Uncertainty {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.variant, rhs.variant) {
            (UncertaintyVariant::Absolute, UncertaintyVariant::Absolute) => Self {
                low: self.low + rhs.low,
                high: self.high + rhs.high,
                precision: self.precision.min(rhs.precision),
                variant: UncertaintyVariant::Absolute,
            },
            (_, _) => panic!("Please convert both uncertainty to its absolute variants apply additive propagation, {:#?} + {:#?}", &self, &rhs),
        }
    }
}

impl Div for Measure {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value / rhs.value,
            uncertainty: self.uncertainty.to_relative(self.value)
                * rhs.uncertainty.to_relative(rhs.value),
        }
    }
}
impl Mul for Measure {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value * rhs.value,
            uncertainty: self.uncertainty.to_relative(self.value)
                * rhs.uncertainty.to_relative(rhs.value),
        }
    }
}

impl Mul for Uncertainty {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.variant, rhs.variant) {
            (UncertaintyVariant::Relative, UncertaintyVariant::Relative) => {
                 Self {
                low: self.low + rhs.low,
                high: self.high + rhs.high,
                precision: self.precision.max(rhs.precision),
                variant: UncertaintyVariant::Relative,
            }},
            (_,_) => panic!("Please convert both uncertainty to its relative variant to apply multiplicative propagation")
        }
    }
}
