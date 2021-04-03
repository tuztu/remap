
/*pub trait Table: Sized + Send + Unpin {
    fn struct_name() -> String;
    fn table_name() -> String {
        Self::struct_name().to_snake_case()
    }
    fn fields_name() -> Vec<String>;
    fn bind_args(&self, args: Args) -> Args;
    fn from_mysql_row(row: MySqlRow) -> Result<Self, Error>;
}*/

use std::fmt::Debug;
// use heck::SnakeCase;

pub trait Table: Debug {
    fn table_name() -> String {
        use heck::SnakeCase;
        // let a = format!("{:?}", Debug);
        let a = "aAbc".to_string();
        let a = a.to_snake_case();
        // Self::struct_name().to_snake_case()
        a
    }
}

#[cfg(test)]
mod test {
    use crate::table::Table;

    #[test]
    fn test() {
        let a = Table::table_name();
        println!("{}", a);
    }
}