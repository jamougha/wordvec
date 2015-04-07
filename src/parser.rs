use super::models::{LanguageModel, WordVec};
use self::Token::*;
use super::mayberef::MaybeRef::{self, Val, Ref};
use std::fmt::{Display, Formatter, Error};

pub fn parse(expr: &str, model: &LanguageModel) -> Result<WordVec, String> {
    expression(&mut Tokens::from(expr.chars()), model).map(|w| w.take())
}

#[derive(PartialEq, Eq, Debug)]
enum Token {
    RParen,
    LParen,
    Plus,
    Minus,
    Word(String),
    Invalid(char),
}

impl Display for Token {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let invalid;
        fmt.write_str(match *self {
            RParen => ")",
            LParen => "(",
            Plus => "+",
            Minus => "-",
            Word(ref w) => &**w,
            Invalid(w) => {
                invalid = w.to_string();
                &*invalid
            },
        })
    }
}

fn expression<'a, I>(tokens: &mut Tokens<I>, model: &'a LanguageModel)
    -> Result<MaybeRef<'a, WordVec>, String>
    where I: Iterator<Item = char>
{
    let token = try!(tokens.next().ok_or("An expression may not be empty"));
    match token {
        LParen => {
            let vec = try!(expression(tokens, model));
            let rparen = try!(tokens.next().ok_or("Unbalanced parentheses"));
            if rparen != RParen {
                return Err(format!("Expected ')', found '{}'", rparen))
            }
            rhs(vec, tokens, model)
        },
        Word(word) => {
            let vec = try!(model.get(&*word).ok_or_else(||
                format!("'{}' is not present in the language model", &word)
            ));
            rhs(Ref(vec), tokens, model)
        },
        _ => Err(format!("An expression may not start with '{}'", token)),
    }
}


fn rhs<'a, I>(lhs: MaybeRef<'a, WordVec>, tokens: &mut Tokens<I>, model: &LanguageModel)
    -> Result<MaybeRef<'a, WordVec>, String>
    where I: Iterator<Item = char>
{
    match tokens.peek() {
        Some(&RParen) | None => return Ok(lhs),
        _ => {}
    }

    let token = tokens.next().unwrap();
    match token {
        Plus => Ok(Val(lhs.take() + &*try!(expression(tokens, model)))),
        Minus => Ok(Val(lhs.take() - &*try!(expression(tokens, model)))),
        Word(_) | LParen => Err(format!("'{}' found in invalid position", token)),
        Invalid(s) => Err(format!("'{}' could not be tokenized", s)),
        RParen => unreachable!(),
    }
}

struct Tokens<I> {
    iter: I,
    next_char: Option<Option<char>>,
    next_token: Option<Option<Token>>
}

impl<I> Tokens<I> where I: Iterator<Item = char> {
    fn from(iter: I) -> Tokens<I> {
        Tokens {
            iter: iter,
            next_char: None,
            next_token: None,
        }
    }

    fn get_next_token(&mut self) -> Option<Token> {
        while let Some(' ') = self.peek_char() {
            self.take();
        }

        match self.peek_char() {
            Some('a'...'z') => return Some(self.word()),
            // Some('0'..'9') | Some('.') => return Some(self.number()),
            None => return None,
            _ => {}
        }

        Some(match self.take().unwrap() {
            '+' => Plus,
            '-' => Minus,
            '(' => LParen,
            ')' => RParen,
            c => Invalid(c),
        })
    }

    fn peek(&mut self) -> Option<&Token> {
        if let Some(ref t) = self.next_token {
            return t.as_ref();
        }

        self.next_token = Some(self.get_next_token());
        return self.next_token.as_ref().unwrap().as_ref();
    }

    fn peek_char(&mut self) -> Option<char> {
        match self.next_char {
            Some(x) => x,
            None => {
                let next = self.iter.next();
                self.next_char = Some(next);
                next
            }
        }
    }

    fn take(&mut self) -> Option<char> {
        self.peek_char();
        self.next_char.take().unwrap()
    }

    fn word(&mut self) -> Token {
        let mut token = String::new();
        while let Some('a'...'z') = self.peek_char() {
            token.push(self.take().unwrap());
        }

        Word(token)
    }

    fn single_char(&mut self) -> String {
        let mut token = String::new();
        token.push(self.take().unwrap());
        token
    }

}

impl<I> Iterator for Tokens<I> where I: Iterator<Item = char> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.peek();
        self.next_token.take().unwrap()
    }
}

mod test {
    use super::Tokens;
    use super::Token::*;
    #[test]
    fn single_character() {
        let c = Tokens::from("a".chars()).next().unwrap();
        assert_eq!(c, Word("a".to_string()));
    }

    #[test]
    fn single_token() {
        let s = Tokens::from("abc".chars()).next().unwrap();
        assert_eq!(s, Word("abc".to_string()));
    }

    #[test]
    fn two_tokens() {
        let s = Tokens::from("abc def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec![Word("abc".to_string()), Word("def".to_string())]);
    }

    #[test]
    fn plus_tokens() {
        let s = Tokens::from("   abc  +def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec![Word("abc".to_string()), Plus, Word("def".to_string())]);
    }

        #[test]
    fn bracket_tokens() {
        let s = Tokens::from(" ()  (abc  +)def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec![LParen, RParen, LParen, Word("abc".to_string()),
            Plus, RParen, Word("def".to_string())]);
    }
}