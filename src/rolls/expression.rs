
use nom::combinator::all_consuming;
use nom::Err;
use nom::error::Error;
use super::expr;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Num(i64),
    Dice { times: i64, sides: i64 },

    Grp(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn dice(times: i64, sides: i64) -> Self {
        Self::Dice { times, sides }
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl<'a> TryFrom<&'a str> for Expression {
    type Error = Err<Error<&'a str>>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (_, expr) = all_consuming(expr)(value)?;
        Ok(expr)
    }
}

impl From<i64> for Expression {
    fn from(i: i64) -> Self { Self::Num(i) }
}

impl From<i64> for Box<Expression> {
    fn from(i: i64) -> Self {
        Expression::from(i).boxed()
    }
}