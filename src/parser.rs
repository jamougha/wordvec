use super::models::{LanguageModel, WordVec};

pub fn parse(expr: &str, model: &LanguageModel) -> WordVec {
    expression(&mut Tokens::from(expr.chars()), model)
}

#[derive(PartialEq, Eq, Debug)]
enum Token {
    RParen,
    LParen,
    Plus,
    Minus,
    Word(String),
}

use self::Token::*;

fn expression<I>(tokens: &mut Tokens<I>, model: &LanguageModel) -> WordVec
    where I: Iterator<Item = char>
{
    let token = tokens.next().expect("An expression may not be empty");
    match token {
        LParen => {
            let vec = expression(tokens, model);
            assert_eq!(RParen, tokens.next().unwrap());
            rhs(vec, tokens, model)
        },
        Word(word) => {
            let vec = model.get(&*word).unwrap_or_else(||
                panic!("'{}' is not present in the language model", &word)
            ).clone();
            rhs(vec, tokens, model)
        },
        _ => panic!("An expression may not start with {:?}", token),
    }
}

fn rhs<I>(lhs: WordVec, tokens: &mut Tokens<I>, model: &LanguageModel) -> WordVec
    where I: Iterator<Item = char>
{
    match tokens.peek() {
        Some(&RParen) | None => return lhs,
        _ => {}
    }

    match tokens.next().unwrap() {
        Plus => lhs + &expression(tokens, model),
        Minus => lhs - &expression(tokens, model),
        Word(word) => panic!("'{}' found in invalid position", word),
        _ => unreachable!()
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
            c => panic!("{} was not expected", c)
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