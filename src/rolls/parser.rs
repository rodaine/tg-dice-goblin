extern crate nom;

use super::Expression;

use nom::{
    IResult,
    error::ParseError,
    branch::alt,
    bytes::complete::tag,
    multi::many0,
    character::complete::{
        char,
        digit1,
        multispace0,
        one_of,
    },
    combinator::{
        map,
        map_res,
        opt,
    },
    sequence::{
        delimited,
        pair,
        separated_pair,
        tuple,
    },
};

/*

expr    -> factor ( ( "-" | "+" ) factor )* ;
factor  -> primary ( ( "/" | "*" ) primary )* ;
primary -> dice | number | group ;
group   -> "(" expr ")" ;
dice    -> INT("d" | "D")INT | ("d" | "D")INT ;
number  -> -INT | INT ;

*/


fn int(input: &str) -> IResult<&str, i64> {
    map_res(digit1, str::parse)(input)
}

fn number(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((opt(tag("-")), int)),
        |(neg, num)| match neg {
            Some(_) => (-num).into(),
            None => num.into(),
        },
    )(input)
}

fn dice(input: &str) -> IResult<&str, Expression> {
    map(
        separated_pair(opt(int), one_of("dD"), int),
        |(times, sides)| Expression::dice(times.unwrap_or(1), sides),
    )(input)
}

fn group(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(char('('), expr, char(')')),
        |e| Expression::Grp(e.boxed()),
    )(input)
}

fn primary(input: &str) -> IResult<&str, Expression> { ws(alt((dice, number, group)))(input) }

fn factor(input: &str) -> IResult<&str, Expression> {
    let (rem, (lhs, rhss)) = pair(
        primary,
        many0(pair(
            one_of("/*"),
            primary,
        )),
    )(input)?;

    let out = rhss.into_iter().fold(lhs, |out, (o, rhs)| match o {
        '/' => Expression::Div(out.boxed(), rhs.boxed()),
        '*' => Expression::Mul(out.boxed(), rhs.boxed()),
        _ => unreachable!(),
    });

    Ok((rem, out))
}

pub(super) fn expr(input: &str) -> IResult<&str, Expression> {
    let (rem, (lhs, rhss)) = pair(
        factor,
        many0(pair(
            one_of("-+"),
            factor,
        )),
    )(input)?;

    let out = rhss.into_iter().fold(lhs, |out, (o, rhs)| match o {
        '-' => Expression::Sub(out.boxed(), rhs.boxed()),
        '+' => Expression::Add(out.boxed(), rhs.boxed()),
        _ => unreachable!(),
    });

    Ok((rem, out))
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        multispace0,
        inner,
        multispace0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;
    use nom::error::{Error, ErrorKind};

    #[test]
    fn test_int() {
        assert_eq!(Ok(("", 123)), int("123"));

        let big = u64::MAX.to_string();
        assert_eq!(
            Err(Err::Error(Error::new(big.as_ref(), ErrorKind::MapRes))),
            int(big.as_ref()));
    }

    #[test]
    fn test_dice() {
        assert_eq!(Ok(("", Expression::dice(123, 456))), dice("123d456"));
        assert_eq!(Ok(("", Expression::dice(123, 456))), dice("123D456"));
    }

    #[test]
    fn test_number() {
        assert_eq!(Ok(("", Expression::Num(123))), number("123"));
        assert_eq!(Ok(("", Expression::Num(-456))), number("-456"));
    }

    #[test]
    fn test_expr() {
        let tests = [
            ("123", 123.into()),
            ("-456", (-456).into()),
            ("123d456", Expression::dice(123, 456)),
            ("2D4", Expression::dice(2, 4)),
            ("d20", Expression::dice(1, 20)),
            ("D8", Expression::dice(1, 8)),
            ("(123)", Expression::Grp(123.into())),
            ("    (    -456)", Expression::Grp((-456).into())),
            ("3 * -4", Expression::Mul(3.into(), (-4).into())),
        ];

        for (input, ex) in tests {
            assert_eq!(Ok(("", ex)), expr(input));
        }
    }
}