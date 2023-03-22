use itertools::Itertools;
use std::{
    borrow::{BorrowMut, Cow},
    io::{self, Write},
    iter::{Enumerate, Peekable},
    mem,
    str::Chars,
};

fn main() -> io::Result<()> {
    repl()
}

fn repl() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    // let mut stdin = BufReader::new(io::stdin());
    let mut line: String = "".to_owned();

    loop {
        print!("user> ");
        stdout.flush()?;
        stdin.read_line(&mut line)?;

        if line.trim() == "exit" {
            break;
        }

        let source = mem::take(&mut line);
        let mut parser = Parser::new(&source);
        let exprs = parser.parse();

        println!("{exprs:?}");
    }

    Ok(())
}

struct Parser<'a> {
    _source: Cow<'a, str>,
    stream: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: Cow::from(source),
            stream: source.chars().enumerate().peekable(),
        }
    }

    pub fn parse(&mut self) -> Vec<Expression> {
        let mut exprs = vec![];
        while let Some(expr) = self.parse_next() {
            exprs.push(expr);
        }

        exprs
    }

    pub fn parse_next(&mut self) -> Option<Expression> {
        #[inline]
        fn some_expr<T>(expr: T) -> Option<Expression>
        where
            Expression: From<T>,
        {
            Some(Expression::from(expr))
        }

        while let Some((_i, char)) = self.stream.next() {
            match char {
                // skip whitespace
                char if char.is_whitespace() => {}
                // KeyWord
                '\'' => return some_expr(KeyWord::SingleQuote),
                '`' => return some_expr(KeyWord::BackTick),
                ',' => return some_expr(KeyWord::Comma),
                ':' => return some_expr(KeyWord::Colon),
                // Delimiter
                '(' => return some_expr(Delimiter::LParen),
                ')' => return some_expr(Delimiter::RParen),
                '{' => return some_expr(Delimiter::LBrace),
                '}' => return some_expr(Delimiter::RBrace),
                '[' => return some_expr(Delimiter::LBracket),
                ']' => return some_expr(Delimiter::RBracket),
                // String
                '"' => return some_expr(self.parse_string()),
                // Comment
                ';' => return Some(self.parse_comment()),
                // Other
                char => return Some(self.parse_other(char)),
            }
        }

        None
    }

    fn parse_string(&mut self) -> Primitive {
        let mut previous_char = ' ';
        let xs: String = self
            .stream
            .borrow_mut()
            .take_while(|(_, char)| {
                let is_terminated = *char == '"' && previous_char != '\\';
                previous_char = *char;

                !is_terminated
            })
            .map(|(_, char)| char)
            .collect();

        Primitive::from(xs)
    }

    fn parse_comment(&mut self) -> Expression {
        let mut previous_char = ' ';
        let mut xs: String = self
            .stream
            .borrow_mut()
            .skip_while(|(_, char)| *char == ';')
            // capture comment, removing carrige return on windows
            .take_while(|(_, char)| {
                #[cfg(windows)]
                let is_terminated = previous_char == '\r' && *char == '\n';
                #[cfg(not(windows))]
                let is_terminated = *char == '\n';

                previous_char = *char;

                !is_terminated
            })
            .map(|(_, char)| char)
            .collect();

        #[cfg(windows)]
        xs.pop();

        Expression::Comment(xs)
    }

    fn parse_other(&mut self, first_character: char) -> Expression {
        let xs: String = self
            .stream
            .borrow_mut()
            .peeking_take_while(|(_, char)| !(char.is_whitespace() || is_reserved_char(*char)))
            .map(|(_, char)| char)
            .collect();
        let capture = format!("{first_character}{xs}");

        // consume as primitive
        if let Some(expr) = maybe_parse_primitive(&capture) {
            return expr;
        }

        // otherwise consume as identifier
        Expression::Ident(capture)
    }
}

#[inline]
const fn is_reserved_char(char: char) -> bool {
    matches!(
        char,
        '(' | ')' | '{' | '}' | '[' | ']' | '"' | '\'' | '`' | ',' | ':' | ';' | '?'
    )
}

#[inline]
fn delimiters_match(exprs: &[Expression]) -> bool {
    exprs
        .iter()
        .filter(|expr| matches!(expr, Expression::Delimiter(_)))
        .fold(
            (0, 0, 0),
            |(mut parens, mut braces, mut brackets): (i16, i16, i16), expr| {
                match expr {
                    Expression::Delimiter(delim) => match delim {
                        Delimiter::LParen => parens += 1,
                        Delimiter::RParen => parens -= 1,
                        Delimiter::LBrace => braces += 1,
                        Delimiter::RBrace => braces -= 1,
                        Delimiter::LBracket => brackets += 1,
                        Delimiter::RBracket => brackets -= 1,
                    },
                    _ => unreachable!("we've filtered out any non-delims"),
                }

                (parens, braces, brackets)
            },
        )
        == (0, 0, 0)
}

#[derive(Debug)]
enum Expression {
    Delimiter(Delimiter),
    Primitive(Primitive),
    Ident(String),
    Comment(String),
    KeyWord(KeyWord),
}

impl From<KeyWord> for Expression {
    fn from(value: KeyWord) -> Self {
        Self::KeyWord(value)
    }
}

impl From<Delimiter> for Expression {
    fn from(value: Delimiter) -> Self {
        Self::Delimiter(value)
    }
}

impl From<Primitive> for Expression {
    fn from(value: Primitive) -> Self {
        Self::Primitive(value)
    }
}

#[derive(Debug)]
enum KeyWord {
    SingleQuote,
    BackTick,
    Comma,
    Colon,
}

#[derive(Debug)]
enum Primitive {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

// #[derive(Debug)]
// struct Delimiter {
//     ty: DelimiterType,
//     kind: DelimiterKind,
// }

#[derive(Debug)]
enum Delimiter {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
}

impl From<char> for Delimiter {
    fn from(char: char) -> Self {
        match char {
            '(' => Self::LParen,
            ')' => Self::RParen,
            '{' => Self::LBrace,
            '}' => Self::RBrace,
            '[' => Self::LBracket,
            ']' => Self::RBracket,
            _ => unreachable!(),
        }
    }
}

// impl From<char> for Delimiter {
//     fn from(char: char) -> Self {
//         match char {
//             '(' => Self {
//                 ty: DelimiterType::Open,
//                 kind: DelimiterKind::Paren,
//             },
//             ')' => Self {
//                 ty: DelimiterType::Close,
//                 kind: DelimiterKind::Paren,
//             },
//             '[' => Self {
//                 ty: DelimiterType::Open,
//                 kind: DelimiterKind::Bracket,
//             },
//             ']' => Self {
//                 ty: DelimiterType::Close,
//                 kind: DelimiterKind::Bracket,
//             },
//             '{' => Self {
//                 ty: DelimiterType::Open,
//                 kind: DelimiterKind::Brace,
//             },
//             '}' => Self {
//                 ty: DelimiterType::Close,
//                 kind: DelimiterKind::Brace,
//             },
//             _ => unreachable!(),
//         }
//     }
// }

// #[derive(Debug)]
// enum DelimiterType {
//     Open,
//     Close,
// }

// #[derive(Debug)]
// enum DelimiterKind {
//     Paren,
//     Brace,
//     Bracket,
// }

impl From<i64> for Primitive {
    #[inline]
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Primitive {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Primitive {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for Primitive {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

fn maybe_parse_primitive(value: &str) -> Option<Expression> {
    #[inline]
    fn apply<T>(value: &str, parser: fn(&str) -> Option<T>) -> Option<Expression>
    where
        Primitive: From<T>,
    {
        parser(value)
            .map(Primitive::from)
            .map(|primitive| Expression::Primitive(primitive))
    }

    apply(value, maybe_parse_int)
        .or_else(|| apply(value, maybe_parse_float))
        .or_else(|| apply(value, maybe_parse_bool))
}

#[inline]
fn maybe_parse_int(value: &str) -> Option<i64> {
    value.parse().ok()
}

#[inline]
fn maybe_parse_float(value: &str) -> Option<f64> {
    value.parse().ok()
}

#[inline]
fn maybe_parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

// #[derive(Debug)]
// enum NativeFunction {
//     Add,
//     Sub,
//     Mul,
//     Div,
//     Mod,
//     Print,
// }

// // #[derive(Debug)]
// // struct Function {
// //     id: Box<str>,
// //     definition: fn(Vec<Expression>) -> Expression,
// // }

// fn lex(source: &str) -> Vec<Token> {
//     let lex_one = |value: &str| {
//         lex_paren(value)
//             .or_else(|| lex_native_function(value))
//             .or_else(|| lex_primitive(value))
//             .unwrap()
//     };

//     source
//         .replace('(', " ( ")
//         .replace(')', " ) ")
//         .split_whitespace()
//         .map(lex_one)
//         .collect()
// }

// fn lex_paren(value: &str) -> Option<Token> {
//     let some_token = |kind| Some(Token::Paren(kind));

//     match value {
//         "(" => some_token(DelimiterKind::Open),
//         ")" => some_token(DelimiterKind::Close),
//         _ => None,
//     }
// }

// fn lex_native_function(value: &str) -> Option<Token> {
//     let some_token = |kind| Some(Token::NativeFunction(kind));

//     match value {
//         "+" => some_token(NativeFunction::Add),
//         "-" => some_token(NativeFunction::Sub),
//         "*" => some_token(NativeFunction::Mul),
//         "/" => some_token(NativeFunction::Div),
//         "%" => some_token(NativeFunction::Mod),
//         "print" => some_token(NativeFunction::Print),
//         _ => None,
//     }
// }

// fn add(expressions: Vec<Expression>) -> Expression {
//     Expression::Primitive(Primitive::Int(
//         expressions
//             .into_iter()
//             .map(|expr| match expr {
//                 Expression::Primitive(Primitive::Int(value)) => value,
//                 _ => todo!(),
//             })
//             .sum(),
//     ))
// }

// fn sub(expressions: Vec<Expression>) -> Expression {
//     Expression::Primitive(Primitive::Int({
//         let mut expr_iter = expressions.into_iter();
//         let acc = match expr_iter.next().unwrap() {
//             Expression::Primitive(Primitive::Int(value)) => value,
//             _ => todo!(),
//         };

//         expr_iter.fold(acc, |acc, expr| match expr {
//             Expression::Primitive(Primitive::Int(value)) => acc - value,
//             _ => todo!(),
//         })
//     }))
// }

#[inline]
const fn newline() -> &'static str {
    #[cfg(windows)]
    return "\r\n";

    return "\n";
}
