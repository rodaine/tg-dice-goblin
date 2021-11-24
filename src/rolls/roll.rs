use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use rand::prelude::*;
use rand::distributions::Uniform;
use super::Expression;
use nom::Err;
use nom::error::Error;

#[derive(Debug)]
pub enum Roll {
    Num(i64),
    Dice(Vec<i64>),
    ManyDice(BTreeMap<i64, i64>),
    TooManyDice(i64),
    Grp(Box<Roll>),
    Add(Box<Roll>, Box<Roll>),
    Sub(Box<Roll>, Box<Roll>),
    Mul(Box<Roll>, Box<Roll>),
    Div(Box<Roll>, Box<Roll>),
}

impl Roll {
    pub fn value(&self) -> i64 {
        use Roll::*;

        match self {
            Num(i) => *i,
            Dice(v) => v.iter().sum(),
            ManyDice(m) => m.iter().fold(0, |s, (val, times)| s + (*val) * (*times)),
            TooManyDice(i) => *i,
            Grp(expr) => expr.value(),
            Add(lhs, rhs) => lhs.value() + rhs.value(),
            Sub(lhs, rhs) => lhs.value() - rhs.value(),
            Mul(lhs, rhs) => lhs.value() * rhs.value(),
            Div(lhs, rhs) => {
                let r = rhs.value();
                if r == 0 {
                    return 0;
                }
                lhs.value() / r
            }
        }
    }
}

impl Roll {
    fn roll_iter(times: i64, sides: i64) -> impl IntoIterator<Item=i64> {
        Uniform::from(1..=sides)
            .sample_iter(thread_rng())
            .take(times as usize)
    }

    fn new_roll(times: i64, sides: i64) -> Self {
        let mut v = Vec::with_capacity(times as usize);

        for n in Roll::roll_iter(times, sides) {
            v.push(n);
        }

        Self::Dice(v)
    }

    fn roll_many(times: i64, sides: i64) -> Self {
        let mut m = BTreeMap::new();

        for n in Roll::roll_iter(times, sides) {
            *m.entry(n).or_insert(0) += 1;
        }

        Self::ManyDice(m)
    }

    fn roll_too_many(times: i64, sides: i64) -> Self {
        let n = Roll::roll_iter(times, sides).into_iter().sum();
        Self::TooManyDice(n)
    }
}

impl From<&Expression> for Roll {
    fn from(expr: &Expression) -> Self {
        use Expression::*;

        match expr {
            Num(i) => Self::Num(*i),
            Dice { times, sides } if *times > 20 && *sides > 20 => Self::roll_too_many(*times, *sides),
            Dice { times, sides } if *times > 20 => Self::roll_many(*times, *sides),
            Dice { times, sides } => Self::new_roll(*times, *sides),
            Grp(e) => Self::Grp(e.into()),
            Add(lhs, rhs) => Self::Add(lhs.into(), rhs.into()),
            Sub(lhs, rhs) => Self::Sub(lhs.into(), rhs.into()),
            Mul(lhs, rhs) => Self::Mul(lhs.into(), rhs.into()),
            Div(lhs, rhs) => Self::Div(lhs.into(), rhs.into()),
        }
    }
}

impl From<&Box<Expression>> for Box<Roll> {
    fn from(be: &Box<Expression>) -> Self {
        Box::new(be.as_ref().into())
    }
}

impl<'a> TryFrom<&'a str> for Roll {
    type Error = Err<Error<&'a str>>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Expression::try_from(value).
            map(|e| (&e).into())
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Roll::*;
        match self {
            Num(i) => write!(f, "{}", i),
            Dice(v) => write!(f, "{:?}", v),
            ManyDice(m) => {
                write!(f, "[")?;
                let mut first = true;
                for (k, v) in m {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}:{}", k, v)?;
                }
                write!(f, "]")
            }
            TooManyDice(i) => write!(f, "[{}]", i),
            Grp(expr) => write!(f, "({})", expr),
            Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
            Sub(lhs, rhs) => write!(f, "{} - {}", lhs, rhs),
            Mul(lhs, rhs) => write!(f, "{} * {}", lhs, rhs),
            Div(lhs, rhs) => write!(f, "{} / {}", lhs, rhs),
        }
    }
}