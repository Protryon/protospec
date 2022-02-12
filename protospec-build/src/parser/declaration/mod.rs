use super::*;

mod const_declaration;
use const_declaration::*;

mod ffi_declaration;
use ffi_declaration::*;

mod import_declaration;
use import_declaration::*;

mod type_declaration;
use type_declaration::*;

pub fn parse_declaration(t: &mut TokenIter) -> ParseResult<Declaration> {
    let declaration = match t.peek()? {
        Token::Type => Declaration::Type(parse_type_declaration(t)?),
        Token::Import => Declaration::Import(parse_import_declaration(t)?),
        Token::ImportFfi => Declaration::Ffi(parse_ffi_declaration(t)?),
        Token::Const => Declaration::Const(parse_const_declaration(t)?),
        _ => {
            return Err(ParseError::Unexpected(
                t.expect_any()?,
                "'type', 'import', 'import_ffi', or 'const'".to_string(),
            ))
        }
    };
    t.expect(Token::Semicolon)?;
    Ok(declaration)
}
