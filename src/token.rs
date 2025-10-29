use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum Token {
    Word(String),
    Newline,
}

pub fn tokenize(s: String) -> Vec<Token> {
    s.lines()
        .into_iter()
        .map(|line| {
            line.split_whitespace()
                .into_iter()
                .map(|word| Token::Word(word.to_string()))
                .collect::<Vec<_>>()
        })
        .into_iter()
        .intersperse(vec![Token::Newline])
        .flatten()
        .collect::<Vec<Token>>()
}

pub fn construct_string(tokens: Vec<Token>) -> String {
    tokens
        .split(|token| matches!(token, Token::Newline))
        .map(|chunk| {
            chunk
                .iter()
                .map(|token| match token {
                    Token::Word(word) => word.to_string(),
                    Token::Newline => panic!("이런 경우는 발생할 수 없음"),
                })
                .join(" ")
        })
        .join("\n")
}
