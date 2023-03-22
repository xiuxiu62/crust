use itertools::Itertools;
use std::{
    borrow::BorrowMut,
    io::{self, Write},
    iter::{Enumerate, Peekable},
    mem,
    str::Chars,
};

fn main() -> io::Result<()> {
    repl()

    // let mut xs = [1, 2, 3, 4, 5].into_iter();
    // let x: Vec<u8> = xs.borrow_mut().take_while(|n| *n != 3).collect();
    // let xs: Vec<u8> = xs.collect();

    // println!("{x:?}");
    // println!("{xs:?}");

    // Ok(())
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
        let exprs = parse(&source);
        println!("{exprs:?}");
    }

    Ok(())
}

fn parse(source: &str) -> Vec<Expression> {
    let mut exprs = vec![];
    fn append<E>(exprs: &mut Vec<Expression>, into_expr: E)
    where
        Expression: From<E>,
    {
        exprs.push(Expression::from(into_expr));
    }

    let mut stream: Peekable<Enumerate<Chars>> = source.chars().enumerate().peekable();
    while let Some((_i, char)) = stream.next() {
        match char {
            // skip whitespace
            char if char.is_whitespace() => {}
            // KeyWord
            '\'' => append(&mut exprs, KeyWord::SingleQuote),
            '`' => append(&mut exprs, KeyWord::BackTick),
            ',' => append(&mut exprs, KeyWord::Comma),
            ':' => append(&mut exprs, KeyWord::Colon),
            // ';' => append(&mut exprs, KeyWord::SemiColon),
            // Delimiter
            '(' => append(&mut exprs, Delimiter::LParen),
            ')' => append(&mut exprs, Delimiter::RParen),
            '{' => append(&mut exprs, Delimiter::LBrace),
            '}' => append(&mut exprs, Delimiter::RBrace),
            '[' => append(&mut exprs, Delimiter::LBracket),
            ']' => append(&mut exprs, Delimiter::RBracket),
            // String
            '"' => {
                let mut previous_char = ' ';
                let xs: String = stream
                    .borrow_mut()
                    .take_while(|(_, char)| {
                        let is_terminated = *char == '"' && previous_char != '\\';
                        previous_char = *char;

                        !is_terminated
                    })
                    .map(|(_, char)| char)
                    .collect();

                append(&mut exprs, Primitive::from(xs))
            }
            // Comment
            ';' => {
                let xs: String = stream
                    .borrow_mut()
                    .skip_while(|(_, char)| *char == ';')
                    // TODO: roll condition into a valid_ident_char method
                    .take_while(|(_, char)| *char != '\n')
                    .map(|(_, char)| char)
                    .collect();

                exprs.push(Expression::Comment(xs));
            }
            // Other
            x => {
                let xs: String = stream
                    .borrow_mut()
                    // TODO: roll condition into a valid_ident_char method
                    .peeking_take_while(|(_, char)| {
                        !(char.is_whitespace() || is_reserved_char(*char))
                    })
                    .map(|(_, char)| char)
                    .collect();
                let capture = format!("{x}{xs}");

                // consume as primitive
                if let Some(expr) = maybe_parse_primitive(&capture) {
                    exprs.push(expr);
                    continue;
                }

                // otherwise consume as identifier
                exprs.push(Expression::Ident(capture));
            }
        }
    }

    exprs
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
    KeyWord(KeyWord),
    Delimiter(Delimiter),
    Primitive(Primitive),
    Ident(String),
    Comment(String),
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
