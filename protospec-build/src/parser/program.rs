use super::*;

pub fn parse_program(t: &mut TokenIter) -> ParseResult<Program> {
    let mut declarations = vec![];
    while t.has_next() {
        declarations.push(parse_declaration(t)?);
    }
    Ok(Program { declarations })
}
