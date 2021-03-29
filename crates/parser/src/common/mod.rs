pub(crate) mod error;

#[cfg(test)]
pub(crate) mod test {
    use logos::Logos;

    use crate::token::Token;

    fn tokens(code: &str) -> Vec<Token> {
        Token::lexer(code).collect()
    }

    fn first_token(code: &str) -> Token {
        (tokens(code)[0]).clone()
    }

    pub(crate) fn assert_error(code: &str) {
        assert_token(code, Token::Error);
    }

    pub(crate) fn assert_token(code: &str, token: Token) {
        assert_eq!(first_token(code), token);
    }

    pub(crate) fn assert_empty(code: &str) {
        assert_tokens(code, &[]);
    }

    pub(crate) fn assert_tokens(code: &str, tokens_to_compare: &[Token]) {
        assert_eq!(tokens(code), tokens_to_compare);
    }
}
