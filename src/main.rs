use core::f64;
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    usize,
};

const DEFAULT_PRECISION: usize = 5;

struct Measure {
    value: f64,
    uncertainty: Uncertainty<f64, usize>,
}

impl Measure {
    fn with_no_err(value: f64) -> Measure {
        Measure {
            value,
            uncertainty: Uncertainty::Certain,
        }
    }
    fn with_rel_err(value: f64, err: f64) -> Measure {
        Measure {
            value,
            uncertainty: Uncertainty::Relative(err.abs(), DEFAULT_PRECISION),
        }
    }
    fn with_abs_err(value: f64, err: f64) -> Measure {
        Measure {
            value,
            uncertainty: Uncertainty::Absolute(err.abs(), DEFAULT_PRECISION),
        }
    }
    fn with_precision(&self, precision: usize) -> Self {
        Self {
            value: self.value,
            uncertainty: self.uncertainty.with_precision(precision),
        }
    }
}

enum Uncertainty<T, P> {
    Absolute(T, P),
    Relative(T, P),
    Certain,
}

impl Uncertainty<f64, usize> {
    fn unwrap(&self) -> f64 {
        match self {
            Uncertainty::Absolute(v, _) => *v,
            Uncertainty::Relative(v, _) => *v,
            Uncertainty::Certain => 0.,
        }
    }
    fn with_precision(&self, precision: usize) -> Uncertainty<f64, usize> {
        match self {
            Uncertainty::Absolute(v, _) => Uncertainty::Absolute(*v, precision),
            Uncertainty::Relative(v, _) => Uncertainty::Relative(*v, precision),
            Uncertainty::Certain => Uncertainty::Absolute(0., precision),
        }
    }
    fn precision(&self) -> usize {
        match self {
            Uncertainty::Absolute(_, p) => *p,
            Uncertainty::Relative(_, p) => *p,
            Uncertainty::Certain => DEFAULT_PRECISION,
        }
    }
    fn to_absolute(&self, value: f64) -> Uncertainty<f64, usize> {
        match self {
            Uncertainty::Absolute(v, p) => Uncertainty::Absolute(*v, *p),
            Uncertainty::Relative(v, p) => {
                Uncertainty::Absolute((v / 100.) * value.abs() as f64, *p)
            }
            Uncertainty::Certain => Uncertainty::Absolute(0., DEFAULT_PRECISION),
        }
    }
    fn to_relative(&self, value: f64) -> Uncertainty<f64, usize> {
        match self {
            Uncertainty::Absolute(v, p) => {
                Uncertainty::Relative((v / value.abs() as f64) * 100., *p)
            }
            Uncertainty::Relative(v, p) => Uncertainty::Relative(*v, *p),
            Uncertainty::Certain => Uncertainty::Relative(0., DEFAULT_PRECISION),
        }
    }
}

fn main() {
    let a = Measure::with_no_err(1.23).with_precision(1) + Measure::with_abs_err(2.0, 0.1);
    let b = Measure::with_no_err(1.25) - Measure::with_rel_err(2.0, 50.).with_precision(3);
    let c = Measure::with_no_err(1.28).with_precision(1) * Measure::with_abs_err(2.0, 0.1);
    let d = Measure::with_no_err(12.) / Measure::with_abs_err(2.91, 0.1).with_precision(2);

    println!("a \t= {}", &a);
    println!("b \t= {}", &b);
    println!("c \t= {}", &c);
    println!("d \t= {}", &d);
    println!("a + d \t= {}", a + d);
    println!("b * c \t= {}", b * c);
}

impl Display for Measure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:>pad$.precision$} ± {:>pad$.precision$} \t:= ({:>pad$.precision$}, {:>pad$.precision$})",
            self.value,
            self.uncertainty,
            self.value - self.uncertainty.to_absolute(self.value).unwrap(),
            self.value + self.uncertainty.to_absolute(self.value).unwrap(),
            precision = self.uncertainty.precision(),
            pad = 0
        )
    }
}

impl Display for Uncertainty<f64, usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Uncertainty::Absolute(..) => {
                write!(
                    f,
                    "{:.precision$}",
                    self.unwrap(),
                    precision = self.precision()
                )
            }
            Uncertainty::Relative(..) => {
                write!(
                    f,
                    "{:.precision$} %",
                    self.unwrap(),
                    precision = self.precision()
                )
            }
            Uncertainty::Certain => write!(
                f,
                "{:.precision$}",
                self.unwrap(),
                precision = self.precision()
            ),
        }
    }
}

impl Add for Measure {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            uncertainty: Uncertainty::Absolute(
                self.uncertainty.to_absolute(self.value).unwrap()
                    + rhs.uncertainty.to_absolute(rhs.value).unwrap(),
                self.uncertainty
                    .precision()
                    .min(rhs.uncertainty.precision()),
            ),
        }
    }
}
impl Sub for Measure {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value - rhs.value,
            uncertainty: Uncertainty::Absolute(
                self.uncertainty.to_absolute(self.value).unwrap()
                    + rhs.uncertainty.to_absolute(rhs.value).unwrap(),
                self.uncertainty
                    .precision()
                    .min(rhs.uncertainty.precision()),
            ),
        }
    }
}
impl Div for Measure {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value / rhs.value,
            uncertainty: Uncertainty::Relative(
                self.uncertainty.to_relative(self.value).unwrap()
                    + rhs.uncertainty.to_relative(rhs.value).unwrap(),
                self.uncertainty
                    .precision()
                    .min(rhs.uncertainty.precision()),
            ),
        }
    }
}
impl Mul for Measure {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value * rhs.value,
            uncertainty: Uncertainty::Relative(
                self.uncertainty.to_relative(self.value).unwrap()
                    + rhs.uncertainty.to_relative(rhs.value).unwrap(),
                self.uncertainty
                    .precision()
                    .min(rhs.uncertainty.precision()),
            ),
        }
    }
}
