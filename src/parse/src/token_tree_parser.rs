use super::*;

impl<'a> Parser<'a> {
    pub fn parse_tt(&mut self) -> ParseResult<'a, TokenTree> {
        match self.safe_peek()?.kind {
            TokenKind::OpenParen => {}
            TokenKind::OpenBrace => {}
            TokenKind::OpenSqBracket => {}
            _ => todo!(),
        }
        todo!()
    }

    pub fn parse_tt_group(&mut self) -> ParseResult<'a, TokenTreeGroup> {
        match self.safe_peek()?.kind {
            TokenKind::OpenParen => {}
            TokenKind::OpenBrace => {}
            TokenKind::OpenSqBracket => {}
            _ => todo!(),
        }
        todo!()
    }
}
