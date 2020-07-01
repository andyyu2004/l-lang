#[derive(Default)]
pub struct Driver {}

impl Driver {
    pub fn lex(&self, src: &str) {
        println!("{:?}", crate::lexer::tokenize(src).collect::<Vec<_>>());
    }

    pub fn parse(&self, src: &str) {
        let tokens = self.lex(src);
    }

    pub fn gen_ir(&self, src: &str) {
        let ast = self.parse(src);
    }

    pub fn gen_tir(&self, src: &str) {
        let ir = self.gen_ir(src);
    }

    pub fn compile(&self, src: &str) {
        let tir = self.gen_tir(src);
    }

    pub fn exec(&self) {
    }
}
