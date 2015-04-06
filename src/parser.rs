use super::models::*;

fn parse(expr: &str, model: &LanguageModel) -> WordVec {
    expression(&mut Tokens::from(expr.chars()), model)
}

fn expression<I>(tokens: &mut Tokens<I>, model: &LanguageModel) -> WordVec
    where I: Iterator<Item = char>
{
    let token = tokens.next().expect("An expression may not be empty");
    match &*token {
        "(" => {
            let vec = expression(tokens, model);
            assert_eq!(")", tokens.next().unwrap());
            rhs(&vec, tokens, model)
        },
        "+" | "-" | ")" => panic!("An expression may not start with {}", token),
        word => {
            let vec = model.get(word).unwrap_or_else(||
                panic!("'{}' is not present in the language model", &word));
            rhs(vec, tokens, model)
        },
    }
}

fn rhs<I>(lhs: &WordVec, tokens: &mut Tokens<I>, model: &LanguageModel) -> WordVec
    where I: Iterator<Item = char>
{
    match tokens.peek()  {
        Some(")") | None => return lhs.clone(),
        _ => {}
    }

    match &*tokens.next().unwrap() {
        "+" => lhs + &expression(tokens, model),
        "-" => lhs - &expression(tokens, model),
        token => panic!("'{}' found in invalid position", token),
    }
}

struct Tokens<I> {
    iter: I,
    next_char: Option<Option<char>>,
    next_token: Option<Option<String>>,
}

impl<I> Tokens<I> where I: Iterator<Item = char> {
    fn from(iter: I) -> Tokens<I> {
        Tokens {
            iter: iter,
            next_char: None,
            next_token: None,
        }
    }

    fn peek(&mut self) -> Option<&str> {
        if let Some(ref t) = self.next_token {
            return t.as_ref().map(|s| &**s);
        }

        loop {
            let token = match self.peek_char() {
                Some('a'...'z') => Some(self.word()),
                Some('+') | Some('-') | Some('(') | Some(')') =>
                    Some(self.single_char()),
                None => None,
                Some(' ') => {
                    self.take();
                    continue
                },
                Some(x) => panic!("{} is not a valid character", x),
            };

            self.next_token = Some(token);
            return self.next_token.as_ref().unwrap().as_ref().map(|s| &**s);
        }
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

    fn word(&mut self) -> String {
        let mut token = String::new();
        while let Some('a'...'z') = self.peek_char() {
            token.push(self.take().unwrap());
        }

        token
    }

    fn single_char(&mut self) -> String {
        let mut token = String::new();
        token.push(self.take().unwrap());
        token
    }

}

impl<I> Iterator for Tokens<I> where I: Iterator<Item = char> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.peek();
        self.next_token.take().unwrap()
    }
}

mod test {
    use super::Tokens;
    #[test]
    fn single_character() {
        let c = Tokens::from("a".chars()).next().unwrap();
        assert_eq!(c, "a");
    }

    #[test]
    fn single_token() {
        let s = Tokens::from("abc".chars()).next().unwrap();
        assert_eq!(s, "abc");
    }

    #[test]
    fn two_tokens() {
        let s = Tokens::from("abc def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec!["abc", "def"]);
    }

    #[test]
    fn plus_tokens() {
        let s = Tokens::from("   abc  +def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec!["abc", "+", "def"]);
    }

        #[test]
    fn bracket_tokens() {
        let s = Tokens::from(" ()  (abc  +)def".chars()).collect::<Vec<_>>();
        assert_eq!(s, vec!["(", ")", "(", "abc", "+", ")", "def"]);
    }
}