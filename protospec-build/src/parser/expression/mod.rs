use super::*;

mod or;
use or::*;

mod and;
use and::*;

mod bit_or;
use bit_or::*;

mod bit_xor;
use bit_xor::*;

mod bit_and;
use bit_and::*;

mod eq;
use eq::*;

mod rel;
use rel::*;

mod shift;
use shift::*;

mod add;
use add::*;

mod multiply;
use multiply::*;

mod cast;
use cast::*;

mod unary;
use unary::*;

mod array_index;
use array_index::*;

mod primary;
use primary::*;

pub fn parse_expression(t: &mut TokenIter) -> ParseResult<Expression> {
    let expr = parse_or_expression(t)?;
    if t.eat(Token::Question).is_some() {
        let if_true = parse_expression(t)?;
        t.expect(Token::Colon)?;
        let if_false = parse_or_expression(t)?;
        Ok(Expression::Ternary(TernaryExpression {
            span: *expr.span() + *if_false.span(),
            condition: Box::new(expr),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }))
    } else {
        Ok(expr)
    }
}
