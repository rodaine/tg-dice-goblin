use std::usize;
use std::str::FromStr;
use crate::roll::Roll;
use std::panic::catch_unwind;

pub fn parse(expr: &str) -> Option<Roll> {
    catch_unwind(|| {
        expression
            .parse(expr)
            .map(|(_, d)| d)
            .ok()
    }).unwrap_or(None)
}

type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: 'a,
            F: Fn(Output) -> NewOutput + 'a,
    {
        BoxedParser::new(_map(self, map_fn))
    }

    fn pred<F>(self, pred_fn: F) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a,
            F: Fn(&Output) -> bool + 'a,
    {
        BoxedParser::new(_pred(self, pred_fn))
    }
}

impl<'a, F, Output> Parser<'a, Output> for F
    where
        F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

struct BoxedParser<'a, Output> {
    parser: Box<dyn Parser<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    fn new<P>(parser: P) -> Self
        where
            P: Parser<'a, Output> + 'a,
    {
        Self {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

fn _map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        F: Fn(A) -> B,
{
    move |input|
        parser.parse(input)
            .map(|(next_input, result)| (next_input, map_fn(result)))
}

fn _pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
        F: Fn(&A) -> bool,
{
    move |input| {
        match parser.parse(input) {
            Ok((next_input, value)) if predicate(&value) => Ok((next_input, value)),
            _ => Err(input),
        }
    }
}

fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2.parse(next_input)
                .map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}

fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
    where
        R1: 'a,
        R2: 'a,
        P1: Parser<'a, R1> + 'a,
        P2: Parser<'a, R2> + 'a,
{
    pair(parser1, parser2).map(|(left, _)| left)
}

fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
    where
        R1: 'a,
        R2: 'a,
        P1: Parser<'a, R1> + 'a,
        P2: Parser<'a, R2> + 'a,
{
    pair(parser1, parser2).map(|(_, right)| right)
}

fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }
}

fn match_operator<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    right(space, match_literal(expected))
}

fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}

fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();

        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input);
        }

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }

        Ok((input, result))
    }
}


fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(c) => Ok((&input[c.len_utf8()..], c)),
        _ => Err(input),
    }
}

fn digit(input: &str) -> ParseResult<char> {
    any_char.pred(|c| c.is_digit(10)).parse(input)
}

fn whitespace(input: &str) -> ParseResult<char> {
    any_char.pred(|c| c.is_whitespace()).parse(input)
}

fn space(input: &str) -> ParseResult<()> {
    zero_or_more(whitespace).map(|_| ()).parse(input)
}

fn number(input: &str) -> ParseResult<usize> {
    one_or_more(digit).map(|chars| {
        let s: String = chars.into_iter().collect();
        match usize::from_str(&s) {
            Ok(n) => n,
            _ => 0,
        }
    }).parse(input)
}

fn non_zero(input: &str) -> ParseResult<usize> {
    number.pred(|&n| n > 0).parse(input)
}

fn either<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parser<'a, A>
    where
        P1: Parser<'a, A>,
        P2: Parser<'a, A>,
{
    move |input| match parser1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => parser2.parse(input),
    }
}

fn expression(input: &str) -> ParseResult<Roll> {
    subtraction(input)
}

fn binary_op<'a, P1, P2>(lhs: P1, op: &'static str, rhs: P2) -> impl Parser<'a, (Roll, Vec<Roll>)>
    where
        P1: Parser<'a, Roll> + 'a,
        P2: Parser<'a, Roll> + 'a,
{
    pair(lhs,
         zero_or_more(right(
             match_operator(op),
             rhs)))
}

fn subtraction(input: &str) -> ParseResult<Roll> {
    binary_op(addition, "-", addition)
        .map(|(lhs, rhs)| {
            let mut roll = lhs;
            for r in rhs {
                roll = Roll::sub(roll, r);
            }
            roll
        }).parse(input)
}

fn addition(input: &str) -> ParseResult<Roll> {
    binary_op(division, "+", division)
        .map(|(lhs, rhs)| {
            let mut roll = lhs;
            for r in rhs {
                roll = Roll::add(roll, r);
            }
            roll
        }).parse(input)
}

fn division(input: &str) -> ParseResult<Roll> {
    binary_op(multiplication, "/", multiplication)
        .map(|(lhs, rhs)| {
            let mut roll = lhs;
            for r in rhs {
                roll = Roll::div(roll, r);
            }
            roll
        }).parse(input)
}

fn multiplication(input: &str) -> ParseResult<Roll> {
    binary_op(primary, "*", primary)
        .map(|(left, right)| {
            let mut roll = left;
            for r in right {
                roll = Roll::mul(roll, r);
            }
            roll
        }).parse(input)
}

fn primary(input: &str) -> ParseResult<Roll> {
    right(
        space,
        either(
            group,
            either(dice, static_num)))
        .parse(input)
}

fn group(input: &str) -> ParseResult<Roll> {
    right(
        match_literal("("),
        left(
            expression,
            match_literal(")"),
        ),
    ).map(Roll::grp).parse(input)
}

fn static_num(input: &str) -> ParseResult<Roll> {
    _map(number, Roll::Static).parse(input)
}

fn dice(input: &str) -> ParseResult<Roll> {
    use Roll::Dice;

    let (next_input, times) = match number(input) {
        Ok(res) if res.1 == 0 => return Err(input),
        Ok(res) => res,
        Err(e) => (e, 1),
    };

    let match_d = either(match_literal("d"), match_literal("D"));
    match right(match_d, non_zero).parse(next_input) {
        Ok((final_input, sides)) => Ok((final_input, Dice { sides, times })),
        _ => Err(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_char() {
        assert_eq!(any_char("abc"), Ok(("bc", 'a')));
        assert_eq!(any_char(""), Err(""));
    }

    #[test]
    fn test_match_literal() {
        let parser = match_literal("ab");
        assert_eq!(parser.parse("abc"), Ok(("c", ())));
        assert_eq!(parser.parse("foo"), Err("foo"));
    }

    #[test]
    fn test_pair() {
        let parser = pair(match_literal("a"), match_literal("bc"));
        assert_eq!(parser.parse("abcd"), Ok(("d", ((), ()))));
        assert_eq!(parser.parse("aeiou"), Err("eiou")); // will consume the first of the pair
        assert_eq!(parser.parse("xyz"), Err("xyz"));
    }

    #[test]
    fn test_map() {
        let parser = _map(any_char, |c| c.is_digit(10));
        assert_eq!(parser.parse("abc"), Ok(("bc", false)));
        assert_eq!(parser.parse("123"), Ok(("23", true)));
        assert_eq!(parser.parse(""), Err(""));
    }

    #[test]
    fn test_left() {
        let parser = left(any_char, match_literal("b"));
        assert_eq!(parser.parse("abc"), Ok(("c", 'a')));
        assert_eq!(parser.parse("aeiou"), Err("eiou"));
        assert_eq!(parser.parse(""), Err(""));
    }

    #[test]
    fn test_right() {
        let parser = right(any_char, any_char);
        assert_eq!(parser.parse("abc"), Ok(("c", 'b')));
        assert_eq!(parser.parse("a"), Err(""));
        assert_eq!(parser.parse(""), Err(""));
    }

    #[test]
    fn test_zero_or_more() {
        let parser = _map(zero_or_more(match_literal("ha")), |x| x.len());
        assert_eq!(parser.parse("hahahaha"), Ok(("", 4)));
        assert_eq!(parser.parse("hats"), Ok(("ts", 1)));
        assert_eq!(parser.parse("hah!"), Ok(("h!", 1)));
        assert_eq!(parser.parse("aha!"), Ok(("aha!", 0)));
    }

    #[test]
    fn test_one_or_more() {
        let parser = _map(one_or_more(match_literal("ha")), |x| x.len());
        assert_eq!(parser.parse("hahahaha"), Ok(("", 4)));
        assert_eq!(parser.parse("hats"), Ok(("ts", 1)));
        assert_eq!(parser.parse("hah!"), Ok(("h!", 1)));
        assert_eq!(parser.parse("aha!"), Err("aha!"));
    }

    #[test]
    fn test_predicate() {
        let parser = _pred(any_char, |c| c.is_alphabetic());
        assert_eq!(parser.parse("abc"), Ok(("bc", 'a')));
        assert_eq!(parser.parse("123"), Err("123"));
        assert_eq!(parser.parse(""), Err(""));
    }

    #[test]
    fn test_digit() {
        assert_eq!(digit("123"), Ok(("23", '1')));
        assert_eq!(digit("abc"), Err("abc"));
        assert_eq!(digit(""), Err(""));
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace(" foo"), Ok(("foo", ' ')));
        assert_eq!(whitespace("abc"), Err("abc"));
        assert_eq!(whitespace(""), Err(""));
    }

    #[test]
    fn test_number() {
        assert_eq!(number("0"), Ok(("", 0)));
        assert_eq!(number("123"), Ok(("", 123)));
        assert_eq!(number("1ab"), Ok(("ab", 1)));
        assert_eq!(number("xyz"), Err("xyz"));
    }

    #[test]
    fn test_non_zero() {
        assert_eq!(non_zero("123"), Ok(("", 123)));
        assert_eq!(non_zero("0"), Err("0"));
        assert_eq!(non_zero("xyz"), Err("xyz"));
    }

    #[test]
    fn test_either() {
        let parser = either(
            match_literal("a"),
            match_literal("x"));

        assert_eq!(parser.parse("abc"), Ok((("bc"), ())));
        assert_eq!(parser.parse("xyz"), Ok((("yz"), ())));
        assert_eq!(parser.parse("123"), Err("123"));
    }

    #[test]
    fn test_dice() {
        use Roll::Dice;

        assert_eq!(dice("4d10"), Ok(("", Dice { sides: 10, times: 4 })));
        assert_eq!(dice("3D3"), Ok(("", Dice { sides: 3, times: 3 })));
        assert_eq!(dice("D4"), Ok(("", Dice { sides: 4, times: 1 })));
        assert_eq!(dice("d2"), Ok(("", Dice { sides: 2, times: 1 })));
        assert_eq!(dice("d0"), Err("d0"));
        assert_eq!(dice("0D5"), Err("0D5"));
    }

    #[test]
    fn test_static_num() {
        use Roll::Static;

        assert_eq!(static_num("123"), Ok(("", Static(123))));
        assert_eq!(static_num("12xyz"), Ok(("xyz", Static(12))));
        assert_eq!(static_num("xyz"), Err("xyz"));
    }

    #[test]
    fn test_primary() {
        use Roll::*;

        assert_eq!(primary("123"), Ok(("", Static(123))));
        assert_eq!(primary("4d10"), Ok(("", Dice { sides: 10, times: 4 })));
        assert_eq!(primary("(456) foo"), Ok((" foo", Roll::grp(Static(456)))));
    }

    #[test]
    fn test_multiply() {
        use Roll::*;

        assert_eq!(multiplication("123 * 456"), Ok(("", Roll::mul(Static(123), Static(456)))));
        assert_eq!(multiplication("d2 * 20d5"), Ok(("", Roll::mul(Dice { times: 1, sides: 2 }, Dice { times: 20, sides: 5 }))));

        let expected = Roll::mul(
            Static(123),
            Roll::grp(Roll::mul(Dice { times: 3, sides: 20 }, Static(456))));
        assert_eq!(multiplication("123 * (3d20 * 456)"), Ok(("", expected)));
    }

    #[test]
    fn test_divide() {
        use Roll::*;

        assert_eq!(division("123/456"), Ok(("", Roll::div(Static(123), Static(456)))));
        assert_eq!(division("d2 / 20d5"), Ok(("", Roll::div(Dice { times: 1, sides: 2 }, Dice { times: 20, sides: 5 }))));

        let expected = Roll::div(
            Static(123),
            Roll::grp(Roll::mul(Dice { times: 3, sides: 20 }, Static(456))));
        assert_eq!(division("123 / (3d20 * 456)"), Ok(("", expected)));
    }

    #[test]
    fn test_massive() {
        let res = parse("d99999999999999999999");
        assert!(res.is_none());
    }
}
