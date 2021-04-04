
pub trait Remap<DB: sqlx::Database>: Sized + Send + Unpin {
    fn table_name() -> String;
    fn bind_fields<'a>(&self) -> crate::arguments::Args<'a, DB>;
    fn decode_row(row: DB::Row) -> Result<Self, anyhow::Error>;
}

pub struct User {

}

impl Remap<sqlx::MySql> for User {
    fn table_name() -> String {
        todo!()
    }

    fn bind_fields<'a>(&self) -> crate::arguments::Args<'a, sqlx::MySql> {
        todo!()
    }

    fn decode_row(row: <sqlx::MySql as sqlx::Database>::Row) -> Result<Self, anyhow::Error> {

        todo!()
    }
}




#[cfg(test)]
mod test {
    use crate::extend::Remap;

    #[test]
    fn test() {
        let a = Remap::table_name();
        println!("{}", a);
    }
}