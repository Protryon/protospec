use crate::ast::*;
use crate::tokenizer::*;

mod error;
pub use error::{ParseError, ParseResult};

mod token_iter;
use token_iter::TokenIter;

mod declaration;
use declaration::*;

mod expression;
use expression::*;

mod types;
use types::*;

mod field;
use field::*;

mod program;
use program::*;

mod transform;
use transform::*;

pub fn parse(script: &str) -> ParseResult<Program> {
    let mut tokens = TokenIter::new(crate::tokenize(script)?);

    parse_program(&mut tokens)
}

fn parse_arguments(t: &mut TokenIter, span: &mut Span) -> ParseResult<Vec<Expression>> {
    let mut arguments = vec![];
    if t.eat(Token::LeftParen).is_some() {
        loop {
            if let Some(token) = t.eat(Token::RightParen) {
                *span = *span + token.span;
                break;
            }
            arguments.push(parse_expression(t)?);
            if t.eat(Token::Comma).is_none() {
                *span = *span + t.expect(Token::RightParen)?;
                break;
            }
        }
    }
    Ok(arguments)
}

fn parse_conditional_clause(t: &mut TokenIter) -> ParseResult<Option<Box<Expression>>> {
    Ok(if t.eat(Token::LeftCurly).is_some() {
        let condition = parse_expression(t)?;
        t.expect(Token::RightCurly)?;
        Some(Box::new(condition))
    } else {
        None
    })
}

fn parse_flags(t: &mut TokenIter) -> ParseResult<Vec<Ident>> {
    let mut out = vec![];
    while t.eat(Token::Plus).is_some() {
        out.push(t.expect_ident()?);
    }
    Ok(out)
}

