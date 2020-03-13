use rand::distributions::{Distribution, Uniform};
use std::ops::Deref;
use std::fmt::{Display, Formatter, Result, Debug};

#[derive(Debug, PartialEq)]
pub enum Roll {
    Static(usize),
    Dice {
        sides: usize,
        times: usize,
    },

    Grp(Box<Roll>),
    Add(Box<Roll>, Box<Roll>),
    Sub(Box<Roll>, Box<Roll>),
    Mul(Box<Roll>, Box<Roll>),
    Div(Box<Roll>, Box<Roll>),
}

impl Roll {
    pub fn mul(lhs: Roll, rhs: Roll) -> Self {
        Roll::Mul(
            Box::new(lhs),
            Box::new(rhs),
        )
    }

    pub fn div(lhs: Roll, rhs: Roll) -> Self {
        Roll::Div(
            Box::new(lhs),
            Box::new(rhs),
        )
    }

    pub fn add(lhs: Roll, rhs: Roll) -> Self {
        Roll::Add(
            Box::new(lhs),
            Box::new(rhs),
        )
    }

    pub fn sub(lhs: Roll, rhs: Roll) -> Self {
        Roll::Sub(
            Box::new(lhs),
            Box::new(rhs),
        )
    }

    pub fn grp(r: Roll) -> Self {
        Roll::Grp(Box::new(r))
    }
}

pub enum RollResult {
    Static(i64),
    Dice(Vec<i64>),
    Grp(Box<RollResult>),
    Add(Box<RollResult>, Box<RollResult>),
    Sub(Box<RollResult>, Box<RollResult>),
    Mul(Box<RollResult>, Box<RollResult>),
    Div(Box<RollResult>, Box<RollResult>),
}

impl RollResult {
    pub fn total(&self) -> i64 {
        use RollResult::*;

        match self {
            Static(n) => *n,
            Dice(ns) => ns.iter().sum(),
            Grp(r) => r.total(),
            Add(lhs, rhs) => lhs.total() + rhs.total(),
            Sub(lhs, rhs) => lhs.total() - rhs.total(),
            Mul(lhs, rhs) => lhs.total() * rhs.total(),
            Div(lhs, rhs) => {
                if rhs.total() == 0 {
                    return 0;
                }
                lhs.total() / rhs.total()
            }
        }
    }
}

impl From<&Roll> for RollResult {
    fn from(roll: &Roll) -> Self {
        use Roll::*;

        match roll {
            Static(n) => RollResult::Static(*n as i64),
            Dice { sides, times } => {
                let die = Uniform::from(1..=*sides as i64);
                let mut rng = rand::thread_rng();

                let rolls = (0..(*times))
                    .map(|_| die.sample(&mut rng))
                    .collect();

                RollResult::Dice(rolls)
            }
            Grp(r) => RollResult::Grp(r.into()),
            Add(lhs, rhs) => RollResult::Add(lhs.into(), rhs.into()),
            Sub(lhs, rhs) => RollResult::Sub(lhs.into(), rhs.into()),
            Mul(lhs, rhs) => RollResult::Mul(lhs.into(), rhs.into()),
            Div(lhs, rhs) => RollResult::Div(lhs.into(), rhs.into()),
        }
    }
}

impl From<&Box<Roll>> for Box<RollResult> {
    fn from(br: &Box<Roll>) -> Self { br.deref().into() }
}

impl From<&Roll> for Box<RollResult> {
    fn from(r: &Roll) -> Self { Box::new(r.into()) }
}

impl Debug for RollResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use RollResult::*;

        match self {
            Static(n) => write!(f, "{:?}", n),
            Dice(n) => write!(f, "{:?}", n),
            Grp(r) => write!(f, "({:?})", r.deref()),
            Add(lhs, rhs) => write!(f, "{:?} + {:?}", lhs.deref(), rhs.deref()),
            Sub(lhs, rhs) => write!(f, "{:?} - {:?}", lhs.deref(), rhs.deref()),
            Mul(lhs, rhs) => write!(f, "{:?} × {:?}", lhs.deref(), rhs.deref()),
            Div(lhs, rhs) => write!(f, "{:?} ÷ {:?}", lhs.deref(), rhs.deref()),
        }
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} = {:?}", self.total(), self)
    }
}