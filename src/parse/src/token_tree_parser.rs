use super::*;

impl<'a> Parser<'a> {
    pub fn parse_tt(&mut self) -> ParseResult<'a, TokenTree> {
        let token = self.safe_peek()?;
        let tt = match token.kind {
            TokenKind::OpenParen => todo!(),
            TokenKind::OpenBrace => todo!(),
            TokenKind::OpenBracket => todo!(),
            _ => TokenTree::Token(token),
        };
        Ok(tt)
    }

    pub fn parse_tt_group(&mut self) -> ParseResult<'a, TokenGroup> {
        match self.safe_peek()?.kind {
            TokenKind::OpenParen => {}
            TokenKind::OpenBrace => {}
            TokenKind::OpenBracket => {}
            _ => todo!(),
        }
        todo!()
    }
}
