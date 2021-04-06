pub trait Remap<DB: sqlx::Database>: Sized + Send + Unpin {
    fn table_name() -> &'static str;
    fn fields_name() -> Vec<&'static str>;
    fn fields_args(&self) -> crate::arguments::Args<DB>;
    fn decode_row(row: DB::Row) -> Result<Self, anyhow::Error>;
}

#[cfg(test)]
mod test {
    // use crate::extend::Remap;
    use crate as remap;

    #[test]
    fn test() {
        // println!("{}", User::table_name());
        // println!("{:?}", User::fields_name());
    }

    #[derive(Debug, Remap)]
    #[remap(sqlx::MySql, table = "my_user")]
    pub struct User {
        id: u32,
        name: String
    }
/*
    impl Remap<sqlx::MySql> for User {
        fn table_name() -> &'static str {
            todo!()
        }

        fn fields_name() -> Vec<&'static str> {
            todo!()
        }

        fn fields_args(&self) -> Args<MySql> {
            Args::new().add(&self.id).add(&self.name)
        }

        fn decode_row(row: <MySql as Database>::Row) -> Result<Self, Error> {
            todo!()
        }
    }
*/

}