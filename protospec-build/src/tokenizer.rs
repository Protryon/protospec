use crate::result::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    String(Vec<u8>),
    Int(String),
    CommentLine(String),
    CommentBlock(String),
    Type,
    Equal,
    As,
    Import,
    Comma,
    From,
    ImportFfi,
    Transform,
    Function,
    Const,
    DotDot,
    Dot,
    Elvis,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Bool,
    Lt,
    Gt,
    Arrow,
    Container,
    LeftSquare,
    RightSquare,
    LeftCurly,
    RightCurly,
    Enum,
    LtEq,
    GtEq,
    Eq,
    Ne,
    Question,
    Colon,
    DoubleColon,
    Semicolon,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Not,
    LeftParen,
    RightParen,
    Cast,
    Or,
    And,
    BitOr,
    BitXor,
    BitAnd,
    Shr,
    Shl,
    ShrSigned,
    BitNot,
    True,
    False,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        match self {
            Ident(s) => write!(f, "{}", s),
            String(s) => write!(f, "\"{}\"", std::string::String::from_utf8_lossy(&s[..])), // todo escapes
            Int(s) => write!(f, "{}", s),
            CommentLine(s) => write!(f, "//{}\n", s),
            CommentBlock(s) => write!(f, "/*{}*/ ", s),
            Type => write!(f, "type "),
            Equal => write!(f, "= "),
            As => write!(f, "as "),
            Import => write!(f, "import "),
            Comma => write!(f, ","),
            From => write!(f, "from "),
            ImportFfi => write!(f, "import_ffi "),
            Transform => write!(f, "transform "),
            Function => write!(f, "function "),
            Const => write!(f, "const "),
            DotDot => write!(f, ".. "),
            Dot => write!(f, ". "),
            Elvis => write!(f, "?: "),
            U8 => write!(f, "u8 "),
            U16 => write!(f, "u16 "),
            U32 => write!(f, "u32 "),
            U64 => write!(f, "u64 "),
            U128 => write!(f, "u128 "),
            I8 => write!(f, "i8 "),
            I16 => write!(f, "i16 "),
            I32 => write!(f, "i32 "),
            I64 => write!(f, "i64 "),
            I128 => write!(f, "i128 "),
            F32 => write!(f, "f32 "),
            F64 => write!(f, "f64 "),
            Bool => write!(f, "bool "),
            Lt => write!(f, "< "),
            Gt => write!(f, "> "),
            Arrow => write!(f, "-> "),
            Container => write!(f, "container "),
            LeftSquare => write!(f, "["),
            RightSquare => write!(f, "]"),
            LeftCurly => write!(f, "{{"),
            RightCurly => write!(f, "}}"),
            Enum => write!(f, "enum "),
            LtEq => write!(f, "<= "),
            GtEq => write!(f, ">= "),
            Eq => write!(f, "== "),
            Ne => write!(f, "!= "),
            Question => write!(f, "?"),
            Colon => write!(f, ":"),
            DoubleColon => write!(f, "::"),
            Semicolon => write!(f, ";"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/ "),
            Mod => write!(f, "%"),
            Not => write!(f, "! "),
            LeftParen => write!(f, "("),
            RightParen => write!(f, ")"),
            Cast => write!(f, ":> "),
            Or => write!(f, "|| "),
            And => write!(f, "&& "),
            BitOr => write!(f, "| "),
            BitXor => write!(f, "^"),
            BitAnd => write!(f, "& "),
            Shr => write!(f, ">> "),
            Shl => write!(f, "<< "),
            ShrSigned => write!(f, ">>> "),
            BitNot => write!(f, "~"),
            True => write!(f, "true "),
            False => write!(f, "false "),
        }
    }
}

fn eat<'a>(input: &'a [u8], wanted: &str) -> Option<&'a [u8]> {
    let wanted = wanted.as_bytes();
    if input.len() < wanted.len() {
        return None;
    }
    if &input[0..wanted.len()] == wanted {
        return Some(&input[wanted.len()..]);
    }
    None
}

fn eat_identifier(input: &[u8]) -> Option<(&[u8], &[u8])> {
    if input.len() == 0 {
        return None;
    }
    if !input[0].is_ascii_alphabetic() && input[0] != b'_' {
        return None;
    }
    let mut i = 1usize;
    while i < input.len() {
        if !input[i].is_ascii_alphanumeric() && input[i] != b'_' && input[i] != b'-' {
            break;
        }
        i += 1;
    }
    Some((&input[0..i], &input[i..]))
}

impl Token {
    fn gobble(input: &[u8]) -> (&[u8], Option<Token>) {
        if input.len() == 0 {
            return (input, None);
        }
        match input[0] {
            x if x.is_ascii_whitespace() => return (&input[1..], None),
            b'"' => {
                let mut i = 1;
                let mut out = vec![];
                while i < input.len() {
                    if input[i] == b'\\' && i < input.len() - 1 {
                        i += 1;
                        if input[i].is_ascii_hexdigit() {
                            if i < input.len() - 1 && input[i + 1].is_ascii_hexdigit() {
                                i += 1;
                                out.push(u8::from_str_radix(std::str::from_utf8(&input[i..i + 2]).unwrap(), 16).unwrap());
                            } else {
                                out.push(u8::from_str_radix(std::str::from_utf8(&input[i..i + 1]).unwrap(), 16).unwrap());
                            }
                        } else {
                            out.push(input[i]);
                        }
                    } else if input[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
                if i == input.len() {
                    return (input, None);
                }
                return (
                    &input[(i + 1)..],
                    Some(Token::String(
                        out,
                    )),
                );
            }
            x if x.is_ascii_digit() => {
                let mut i = 1;
                let mut is_hex = false;
                while i < input.len() {
                    if i == 1 && input[0] == b'0' && input[i] == b'x' {
                        is_hex = true;
                        i += 1;
                        continue;
                    }
                    if is_hex {
                        if !input[i].is_ascii_hexdigit() {
                            break;
                        }
                    } else {
                        if !input[i].is_ascii_digit() {
                            break;
                        }
                    }

                    i += 1;
                }
                return (
                    &input[i..],
                    Some(Token::Int(
                        String::from_utf8(input[0..i].to_vec()).unwrap_or_default(),
                    )),
                );
            }
            b'=' => {
                if let Some(input) = eat(input, "==") {
                    return (input, Some(Token::Eq));
                } else {
                    return (&input[1..], Some(Token::Equal));
                }
            }
            b',' => return (&input[1..], Some(Token::Comma)),
            b';' => return (&input[1..], Some(Token::Semicolon)),
            b'?' => {
                if let Some(input) = eat(input, "?:") {
                    return (input, Some(Token::Elvis));
                } else {
                    return (&input[1..], Some(Token::Question));
                }
            },
            b'[' => return (&input[1..], Some(Token::LeftSquare)),
            b']' => return (&input[1..], Some(Token::RightSquare)),
            b'{' => return (&input[1..], Some(Token::LeftCurly)),
            b'}' => return (&input[1..], Some(Token::RightCurly)),
            b'(' => return (&input[1..], Some(Token::LeftParen)),
            b')' => return (&input[1..], Some(Token::RightParen)),
            b'+' => return (&input[1..], Some(Token::Plus)),
            b'*' => return (&input[1..], Some(Token::Mul)),
            b'%' => return (&input[1..], Some(Token::Mod)),
            b'^' => return (&input[1..], Some(Token::BitXor)),
            b'~' => return (&input[1..], Some(Token::BitNot)),
            b'|' => {
                if let Some(input) = eat(input, "||") {
                    return (input, Some(Token::Or));
                } else {
                    return (&input[1..], Some(Token::BitOr));
                }
            }
            b'/' => {
                if let Some(input) = eat(input, "//") {
                    let eol = input.iter().position(|x| *x == b'\n');
                    let (input, comment) = if let Some(eol) = eol {
                        (&input[(eol + 1)..], &input[..eol])
                    } else {
                        (&input[input.len()..input.len()], &input[..])
                    };
                    return (
                        input,
                        Some(Token::CommentLine(
                            String::from_utf8_lossy(comment).to_string(),
                        )),
                    );
                } else if let Some(input) = eat(input, "/*") {
                    if input.len() == 0 {
                        return (input, None);
                    }
                    let eol = input.windows(2).position(|x| x[0] == b'*' && x[1] == b'/');
                    let (input, comment) = if let Some(eol) = eol {
                        (&input[(eol + 2)..], &input[..eol])
                    } else {
                        (&input[input.len()..input.len()], &input[..])
                    };
                    return (
                        input,
                        Some(Token::CommentBlock(
                            String::from_utf8_lossy(comment).to_string(),
                        )),
                    );
                } else {
                    return (&input[1..], Some(Token::Div));
                }
            }
            b'&' => {
                if let Some(input) = eat(input, "&&") {
                    return (input, Some(Token::And));
                } else {
                    return (&input[1..], Some(Token::BitAnd));
                }
            }
            b'.' => {
                if let Some(input) = eat(input, "..") {
                    return (input, Some(Token::DotDot));
                } else {
                    return (&input[1..], Some(Token::Dot));
                }
            }
            b':' => {
                if let Some(input) = eat(input, ":>") {
                    return (input, Some(Token::Cast));
                } else if let Some(input) = eat(input, "::") {
                    return (input, Some(Token::DoubleColon));
                } else {
                    return (&input[1..], Some(Token::Colon));
                }
            }
            b'<' => {
                if let Some(input) = eat(input, "<=") {
                    return (input, Some(Token::LtEq));
                } else if let Some(input) = eat(input, "<<") {
                    return (input, Some(Token::Shl));
                } else {
                    return (&input[1..], Some(Token::Lt));
                }
            }
            b'>' => {
                if let Some(input) = eat(input, ">=") {
                    return (input, Some(Token::GtEq));
                } else if let Some(input) = eat(input, ">>>") {
                    return (input, Some(Token::ShrSigned));
                } else if let Some(input) = eat(input, ">>") {
                    return (input, Some(Token::Shr));
                } else {
                    return (&input[1..], Some(Token::Gt));
                }
            }
            b'-' => {
                if let Some(input) = eat(input, "->") {
                    return (input, Some(Token::Arrow));
                } else {
                    return (&input[1..], Some(Token::Minus));
                }
            }
            b'!' => {
                if let Some(input) = eat(input, "!=") {
                    return (input, Some(Token::Ne));
                } else {
                    return (&input[1..], Some(Token::Not));
                }
            }
            _ => (),
        }
        if let Some((ident, input)) = eat_identifier(input) {
            let ident = String::from_utf8_lossy(ident).to_string();
            return (
                input,
                Some(match &*ident {
                    "type" => Token::Type,
                    "as" => Token::As,
                    "import" => Token::Import,
                    "import_ffi" => Token::ImportFfi,
                    "i8" => Token::I8,
                    "i16" => Token::I16,
                    "i32" => Token::I32,
                    "i64" => Token::I64,
                    "i128" => Token::I128,
                    "u8" => Token::U8,
                    "u16" => Token::U16,
                    "u32" => Token::U32,
                    "u64" => Token::U64,
                    "u128" => Token::U128,
                    "transform" => Token::Transform,
                    "function" => Token::Function,
                    "const" => Token::Const,
                    "container" => Token::Container,
                    "f32" => Token::F32,
                    "f64" => Token::F64,
                    "enum" => Token::Enum,
                    "bool" => Token::Bool,
                    "from" => Token::From,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Ident(ident),
                }),
            );
        }

        (input, None)
    }
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct Span {
    pub line_start: u64,
    pub line_stop: u64,
    pub col_start: u64,
    pub col_stop: u64,
}

impl PartialEq for Span {
    fn eq(&self, _other: &Span) -> bool {
        true
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line_start == self.line_stop {
            write!(
                f,
                "{}:{}-{}",
                self.line_start, self.col_start, self.col_stop
            )
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.line_start, self.col_start, self.line_stop, self.col_stop
            )
        }
    }
}

impl std::ops::Add for Span {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.line_start == other.line_stop {
            Span {
                line_start: self.line_start,
                line_stop: self.line_stop,
                col_start: self.col_start.min(other.col_start),
                col_stop: self.col_stop.max(other.col_stop),
            }
        } else if self.line_start < other.line_start {
            Span {
                line_start: self.line_start,
                line_stop: other.line_stop,
                col_start: self.col_start,
                col_stop: other.col_stop,
            }
        } else {
            Span {
                line_start: other.line_start,
                line_stop: self.line_stop,
                col_start: other.col_start,
                col_stop: self.col_stop,
            }
        }
    }
}

#[derive(Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl fmt::Display for SpannedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' @ ", self.token.to_string().trim())?;
        self.span.fmt(f)
    }
}

impl fmt::Debug for SpannedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <SpannedToken as fmt::Display>::fmt(self, f)
    }
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>> {
    let mut input = input.as_bytes();
    let mut tokens = vec![];
    let mut index = 064;
    let mut line_no = 1u64;
    let mut line_start = 0u64;
    while input.len() > 0 {
        match Token::gobble(input) {
            (output, Some(token)) => {
                tokens.push(SpannedToken {
                    token,
                    span: Span {
                        line_start: line_no,
                        line_stop: line_no,
                        col_start: index - line_start + 1,
                        col_stop: index - line_start + (input.len() - output.len()) as u64 + 1,
                    },
                });
                index += (input.len() - output.len()) as u64;
                input = output;
            }
            (output, None) => {
                if output.len() == 0 {
                    break;
                } else if output.len() == input.len() {
                    return Err(protospec_err!(
                        "unexpected token '{}' @ {}",
                        String::from_utf8_lossy(&[input[0]]),
                        index
                    ));
                }
                index += (input.len() - output.len()) as u64;
                if input[0] == b'\n' {
                    line_no += 1;
                    line_start = index;
                }
                input = output;
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tokens = tokenize(
            r#"
        test_ident
        "string"
        "str\"ing"
        "str\\ing"
        12345
        -12345
        type
        as
        import
        import_ffi
        i8
        u8
        transform
        function
        const/*

        test block*/container
        f32
        f64
        enum
        true
        false
        bool
        from
        ,;:?[]{}<>?+-/ *%..<=>= = == != ! () // test$
        :> || && | ^ | >> << >>>~ . ?:
        //"#,
        )
        .unwrap();
        let mut output = String::new();
        for SpannedToken { token, .. } in tokens.iter() {
            output += &token.to_string();
        }
        assert_eq!(
            output,
            r#"test_ident"string""str\"ing""str\\ing"12345-12345type as import import_ffi i8 u8 transform function const /*

        test block*/ container f32 f64 enum true false bool from ,;:?[]{}< > ?+-/ *%.. <= >= = == != ! ()// test$
:> || && | ^| >> << >>> ~. ?: //
"#
        );
    }
}
