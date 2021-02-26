use anyhow::Error;
use sql_builder::SqlBuilder;
use sqlx::{MySql, Transaction};

use heck::SnakeCase;

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


#[cfg(test)]
mod test {
    use futures_await_test::async_test;
    use sqlx::{mysql::MySqlRow, Row};

    use crate::init_test::initialize;
    use crate::mysql::*;

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Table)]
    pub struct Example {
        pub id: u64,
        pub name: String
    }


}