//! A simple Lexer
//!
//! A lexer based on the regex crate

#[warn(missing_docs)]

/// Contains the main lexer
pub mod lexer {
    use regex::{Regex, RegexSet};
    use lazy_static::lazy_static;

    /// Represents a Lexer Action mapping a regex representation to a TokenType
    #[derive(Clone)]
    pub struct LexAction<'s, TokenType> {
        /// Regex representation of a token
        pub token:  &'s str,
        /// Function converting a `&str` token to a `TokenType`
        pub action: fn(&str) -> TokenType,
    }

    /// Struct used to generate a Lexer
    ///
    /// It can either be initialised with an array of LexActions, or using the
    /// [push](LexerBuilder::push) method(recommended).
    #[derive(Default)]
    pub struct LexerBuilder<'s, TokenType> {
        /// List of all tokens including conversions used by the resulting Lexer
        pub actions: Vec<LexAction<'s, TokenType>>,
    }

    /// Represents a finished Lexer
    pub struct Lexer<TokenType> {
        regex_set: RegexSet,
        regexes: Vec<Regex>,
        actions: Vec<fn(&str) -> TokenType>,
        data: String,
        curr_pos: usize,
    }

    impl<'s, TokenType> LexerBuilder<'s, TokenType> {
        /// Returns an empty LexerBuilder
        pub fn new() -> Self{
            LexerBuilder{ actions: Vec::new() }
        }

        /// Adds a new token to the LexerBuilder
        ///
        /// token is the regex representation of the string  
        /// action is a method converting the &str representation of the token to a Token
        pub fn push(&mut self, token: &'s str, action: fn(&str) -> TokenType) -> &mut Self {
            self.actions.push(LexAction{ token, action });
            self
        }

        /// Builds a new Lexer from the Actions configured in the Builder
        pub fn build(&self) -> Lexer<TokenType>{
            Lexer{
                regex_set: RegexSet::new(self.actions.iter().map(|a| String::from("^") + &a.token )).unwrap(),
                regexes: self.actions.iter().map(|a| Regex::new(&(String::from("^") + &a.token)).unwrap()).collect(),
                actions: self.actions.iter().map(|a| a.action ).collect(),
                data: String::new(),
                curr_pos: 0,
            }
        }
    }

    impl<TokenType> Lexer<TokenType> {
        /// Resets the parser to the starting state with input data
        pub fn init(&mut self, data: String){
            self.data = data;
            self.curr_pos = 0;
        }

        /// Returns the next Token, or None if no token is found
        pub fn tok(&mut self, skip_ws: bool) -> Option<TokenType> {
            println!("{}", &self.data[self.curr_pos..]);
            if skip_ws {
                lazy_static! {
                    static ref WS: Regex = Regex::new(r"^\s").unwrap();
                }

                let res = WS.find(&self.data[self.curr_pos..]);
                match res {
                    Some(v) => { self.curr_pos = v.end() + self.curr_pos; }
                    None => ()
                };
            };
            println!("{} {}\n", self.curr_pos, &self.data[self.curr_pos..]);

            let matches: Vec<_> = self.regex_set.matches(&self.data[self.curr_pos..]).into_iter().collect();

            if matches.is_empty() {
                return None;
            }

            let mut longest = 0;
            let mut longest_id = 0;

            for m in matches {
                println!("{}", self.curr_pos);
                let length = self.regexes[m].find(&self.data[self.curr_pos..]).unwrap().end() + self.curr_pos;
                if length > longest {
                    longest = length;
                    longest_id = m;
                }
            };

            let token = self.actions[longest_id](&self.data[self.curr_pos..longest]);
            self.curr_pos = longest;
            Some(token)
        }

        /// Returns true if the end of input has been reached.
        pub fn is_eof(&self) -> bool {
            self.curr_pos == self.data.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::lexer::{Lexer, LexerBuilder, LexAction};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[derive(Clone)]
    enum Token1 {
        TokenInt    (i32),
        TokenString (String),
    }

    #[test]
    fn doesnt_panic_array(){
        let _l: Lexer<Token1> = LexerBuilder{
            actions: [LexAction{ token: r"\d+", action: |x: &str| Token1::TokenInt( x.parse::<i32>().unwrap() )}].to_vec(),
        }.build();
    }

    #[test]
    fn doesnt_panic_append(){
        let _l: Lexer<Token1> = LexerBuilder::new()
            .push( r"\d+",          |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .push( r"[a-zA-Z_]\w*", |x: &str| Token1::TokenString(String::from(x)))
            .build();
    }

    #[test]
    fn simple_number_test(){
        let mut l = LexerBuilder::<Token1>::new()
            .push(r"\d+", |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .build();

        l.init(String::from("42"));

        match l.tok(true).unwrap() {
            Token1::TokenInt(v) => { assert!(v == 42);},
            _ => { panic!("Token is not of type int"); },
        }
    }

    #[test]
    fn simple_number_leading_ws(){
        let mut l = LexerBuilder::<Token1>::new()
            .push(r"\d+", |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .build();

        l.init(String::from(" 42"));

        match l.tok(true).unwrap() {
            Token1::TokenInt(v) => { assert!(v == 42, "Expected 42: Actual: {}", v);},
            _ => { panic!("Token is not of type int"); },
        }
    }

    #[test]
    fn two_numbers(){
        let mut l = LexerBuilder::<Token1>::new()
            .push(r"\d+", |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .build();

        l.init(String::from("42 52"));

        match l.tok(true).unwrap() {
            Token1::TokenInt(v) => { assert!(v == 42, "Expected 42: Actual {}", v);},
            _ => { panic!("Token is not of type int"); },
        }
 
        match l.tok(true).unwrap() {
            Token1::TokenInt(v) => { assert!(v == 52);},
            _ => { panic!("Token is not of type int"); },
        }       
    }

    #[test]
    fn many_numbers(){
        let mut l = LexerBuilder::<Token1>::new()
            .push(r"\d+", |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .build();
        
        l.init((0..100).map(|x: i8| x.to_string()).collect::<Vec<String>>().join(" "));

        for i in 0..100 {
            match l.tok(true).unwrap() {
                Token1::TokenInt(v) => { assert!(v == i, "Expected {}: Actual {}", i, v);},
                _ => { panic!("Token is not of type int"); },
            }
        }
    }

    #[test]
    fn test_eof(){
        let mut l = LexerBuilder::<Token1>::new()
            .push(r"\d+", |x: &str| Token1::TokenInt(x.parse::<i32>().unwrap()))
            .build();

        l.init(String::from("42"));

        assert!(!l.is_eof());

        match l.tok(true).unwrap() {
            Token1::TokenInt(v) => { assert!(v == 42, "Expected 42: Actual: {}", v);},
            _ => { panic!("Token is not of type int"); },
        }

        assert!(l.is_eof());
    }
}
