#[cfg(test)]
pub(crate) mod test {
    use logos::Logos;

    use crate::token::Token;

    pub(crate) fn assert_error(code: &str) {
        assert_token(code, Token::Error);
    }

    pub(crate) fn assert_token(code: &str, token: Token) {
        let lexed: Vec<Token> = Token::lexer(code).collect();
        assert_eq!(lexed[0], token);
    }

    pub(crate) fn assert_tokens(code: &str, tokens: &[Token]) {
        let lexed: Vec<Token> = Token::lexer(code).collect();
        assert_eq!(lexed, tokens);
    }
}
