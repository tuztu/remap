use anyhow::Error;
use heck::SnakeCase;
use sql_builder::SqlBuilder;
use sqlx::{MySql, Transaction};

use crate::config::{Args, pool, Table};

/// Insert one row into database
pub async fn insert<T: Table>(t: &T) -> Result<u64, Error> {
    let name = T::struct_name().to_snake_case();
    let fields = T::fields_name();
    let args = fields.iter().map(|_| "?").collect::<Vec<&str>>();
    let sql = SqlBuilder::insert_into(name)
        .fields(fields.as_slice())
        .values(args.as_slice()).sql()?;
    let args = t.bind_args(Args::new());
    let result = sqlx::query_with(sql.as_str(), args.mysql_args())
        .execute(pool())
        .await?;
    Ok(result.rows_affected())
}

/// Insert rows into database with transaction
pub async fn insert_tx<'a, T: Table>(v: &Vec<T>, tx: &mut Transaction<'a, MySql>)
    -> Result<u64, Error> {
    let name = T::struct_name().to_snake_case();
    let fields = T::fields_name();
    let args = fields.iter().map(|_| "?").collect::<Vec<&str>>();
    let mut sql_builder = SqlBuilder::insert_into(name);
    sql_builder.fields(fields.as_slice());
    for _ in 0..v.len() {
        sql_builder.values(args.as_slice());
    }
    let sql = sql_builder.sql()?;

    let mut args = Args::new();
    for x in v {
        args = x.bind_args(args);
    }

    let result = sqlx::query_with(sql.as_str(), args.mysql_args())
        .execute(tx)
        .await?;

    Ok(result.rows_affected())
}

/// insert … on duplicate key update
/// TODO build middle type to pass the args.
/*
pub async fn insert_update_tx<'a, T: Table>(
    v: &Vec<T>, set_fields: &[&str], args: Args, tx: &mut Transaction<'a, MySql>)
    -> Result<(),Error> {

    // Insert begin
    let name = T::struct_name().to_snake_case();
    let fields = T::fields_name();
    let args = fields.iter().map(|_| "?").collect::<Vec<&str>>();
    let mut sql_builder = SqlBuilder::insert_into(name);
    sql_builder.fields(fields.as_slice());
    for _ in 0..v.len() {
        sql_builder.values(args.as_slice());
    }
    let mut insert_args = Args::new();
    for x in v {
        insert_args = x.bind_args(insert_args);
    }

    let mut sql = sql_builder.sql()?;
    sql.push_str(" on duplicate key update ");


    // Insert end
    // Update begin

    Ok()
}
*/
pub async fn update<T: Table>(set_fields: &[&str], and_where_eq: &[&str], args: Args)
    -> Result<u64, Error> {
    let name = T::struct_name();
    let mut sql_builder = SqlBuilder::update_table(name.to_snake_case());
    for x in set_fields {
        sql_builder.set(x, "?");
    }
    for x in and_where_eq {
        sql_builder.and_where_eq(x, "?");
    }
    let sql = sql_builder.sql()?;
    let result = sqlx::query_with(sql.as_str(), args.mysql_args())
        .execute(pool())
        .await?;

    Ok(result.rows_affected())
}

pub async fn delete<T: Table>(and_where_eq: &[&str], args: Args) -> Result<u64, Error> {
    let name = T::struct_name();
    let mut sql_builder = SqlBuilder::delete_from(name.to_snake_case());
    for x in and_where_eq {
        sql_builder.and_where_eq(x, "?");
    }
    let sql = sql_builder.sql()?;
    let result = sqlx::query_with(sql.as_str(), args.mysql_args())
        .execute(pool())
        .await?;

    Ok(result.rows_affected())
}

pub async fn select_one<T>(where_eq: &str, args: Args) -> Result<Option<T>, Error>
    where T: Table {
    let name = T::struct_name();
    let mut sql_builder = SqlBuilder::select_from(name.to_snake_case());
    sql_builder.and_where_eq(where_eq, "?");
    let sql = sql_builder.sql()?;

    // query_with::<MySql, MySqlArguments>
    let output: Option<Result<T, Error>> = sqlx::query_with(sql.as_str(), args.mysql_args())
        .map(|row| T::from_mysql_row(row))
        .fetch_optional(pool())
        .await?;

    let output = match output {
        Some(s) => Some(s?),
        None => None
    };
    Ok(output)
}

pub async fn select_in<T>(where_in: &str, args: Args) -> Result<Vec<T>, Error>
    where T: Table {
    let name = T::struct_name();
    let mut sql_builder = SqlBuilder::select_from(name.to_snake_case());
    let mut v = Vec::with_capacity(args.args_size());
    (0..args.args_size()).for_each(|_|  v.push("?") );
    let sql = sql_builder.and_where_in(where_in, v.as_slice()).sql()?;
    let output: Vec<Result<T, Error>> = sqlx::query_with(sql.as_str(), args.mysql_args())
        .map(|row| T::from_mysql_row(row))
        .fetch_all(pool())
        .await?;

    let mut vec = Vec::with_capacity(output.len());
    for x in output {
        vec.push(x?);
    }

    Ok(vec)
}

/*pub async fn select<T>() -> Result<Vec<T>, Error> where T: Table {
    Ok(vec![])
}*/


#[cfg(test)]
mod test {
    use std::sync::Once;

    use async_std::task::block_on;
    use futures_await_test::async_test;
    use sqlx::{mysql::MySqlRow, Row};
    use sqlx::mysql::MySqlPoolOptions;

    use crate::config::*;
    use crate::mysql::*;

    static INIT: Once = Once::new();
    fn initialize_once() {
        INIT.call_once(|| {
            block_on(async {
                let pool = MySqlPoolOptions::new()
                    .max_connections(5)
                    .connect("mysql://root:123456@localhost:3306/remap")
                    .await.expect("connecting mysql server failed");
                init_pool(pool).await.expect("init pool failed");
            });
        });
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Table)]
    pub struct User {
        pub id: u64,
        pub name: String
    }

    #[async_test]
    pub async fn test_insert () {
        initialize_once();
        let ex = User {
            id: 1,
            name: "李四".to_string()
        };
        delete::<User>(&["id"], Args::new().bind(&ex.id)).await.unwrap();
        let a = insert(&ex).await.unwrap();
        assert_eq!(1, a);
        let output = select_one::<User>("id", Args::new().bind(&ex.id))
            .await.unwrap();
        assert_eq!(&ex, &output.unwrap())
    }

    #[async_test]
    pub async fn test_insert_tx() {
        initialize_once();
        let one = User {
            id: 3,
            name: "黎明".to_string()
        };

        let two = User {
            id:42,
            name: "李四".to_string()
        };
        delete::<User>(&["id"], Args::new().bind(&one.id)).await.unwrap();
        delete::<User>(&["id"], Args::new().bind(&two.id)).await.unwrap();

        let rows = vec![one, two];
        let mut tx = pool().begin().await.unwrap();
        insert_tx(&rows, &mut tx).await.unwrap();
        tx.rollback().await.unwrap();
        // let result = tx.commit().await.unwrap();

    }

    #[async_test]
    pub async fn test_update() {
        initialize_once();
        let ex = User { id: 5, name: "张三".to_string() };
        delete::<User>(&["id"], Args::new().bind(&ex.id)).await.unwrap();
        insert(&ex).await.unwrap();
        let name = "李四".to_string();
        let args = Args::new().bind(&name).bind(&ex.id);
        update::<User>(&["name"], &["id"], args).await.unwrap();
    }

    #[async_test]
    pub async fn test_delete() {
        initialize_once();
        let ex = User { id: 6, name: "Sam".to_string() };
        delete::<User>(&["id"], Args::new().bind(&ex.id)).await.unwrap();
        insert(&ex).await.unwrap();
        let aff = delete::<User>(&["id"], Args::new().bind(&ex.id)).await.unwrap();
        assert_eq!(1, aff);
    }

    #[async_test]
    pub async fn test_select_one() {
        initialize_once();
        let ex = User { id: 7, name: "Sam".to_string() };
        delete::<User>(&["id"], Args::new().bind(&ex.id)).await.unwrap();
        insert(&ex).await.unwrap();
        let result = select_one::<User>("id", Args::new().bind(&ex.id))
            .await.unwrap();
        assert!(result.is_some());
        assert_eq!(&ex, result.as_ref().unwrap());
    }

    #[async_test]
    pub async fn test_select_in() {
        initialize_once();
        let ex1 = User { id: 8, name: "Bob".to_string() };
        let ex2 = User { id: 9, name: "Sam".to_string() };
        let ex3 = User { id: 10, name: "Cat".to_string() };

        delete::<User>(&["id"], Args::new().bind(&ex1.id)).await.unwrap();
        delete::<User>(&["id"], Args::new().bind(&ex2.id)).await.unwrap();
        delete::<User>(&["id"], Args::new().bind(&ex3.id)).await.unwrap();
        insert(&ex1).await.unwrap();
        insert(&ex2).await.unwrap();
        insert(&ex3).await.unwrap();
        let args = Args::new().bind(&ex1.id).bind(&ex2.id).bind(&ex3.id);
        let mut result = select_in::<User>("id", args).await.unwrap();
        result.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(&ex1, result.get(0).unwrap());
        assert_eq!(&ex2, result.get(1).unwrap());
        assert_eq!(&ex3, result.get(2).unwrap());
    }

}
