use sqlparser::dialect::Dialect;

#[derive(Debug, Default)]
pub struct TyrDialect;


impl Dialect for TyrDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        todo!()
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        todo!()
    }
}