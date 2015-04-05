pub struct Tokens<I> {
    iter: I,
    next_char: Option<Option<char>>,
}

impl<I> Tokens<I> where I: Iterator<Item = char> {
    fn from(iter: I) -> Tokens<I> {
        Tokens {
            iter: iter,
            next_char: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
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
        self.peek();
        self.next_char.take().unwrap()
    }

    fn word(&mut self) -> String {
        let mut token = String::new();
        while let Some('a'...'z') = self.peek() {
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
        loop {
            match self.peek() {
                Some(' ') => { self.take(); },
                Some('a'...'z') => return Some(self.word()),
                Some('+') | Some('-') | Some('(') | Some(')') =>
                    return Some(self.single_char()),
                None => return None,
                Some(x) => panic!("{} is not a valid character", x),
            }
        }
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