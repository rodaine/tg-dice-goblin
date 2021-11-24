mod expression;
mod parser;
mod roll;

use nom::Err;
use parser::expr;
use expression::Expression;

pub use roll::Roll;
pub type Error<'a> = Err<nom::error::Error<&'a str>>;

pub fn parse(input: &str) -> Result<Roll, Error> {
    input.try_into()
}