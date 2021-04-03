use std::collections::HashMap;
use std::fmt::{Display, Formatter, Debug};

use anyhow::Error;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use sqlx::{Any, Database, Mssql, MySql, Pool, Postgres, Sqlite, Arguments, IntoArguments};
use sqlx::mysql::{MySqlPoolOptions, MySqlArguments};
use crate::config::{Table, Args};
use sql_builder::SqlBuilder;
use std::any::TypeId;
use sqlx::query::Query;

static POOLS: OnceCell<HashMap<String, DbPool>> = OnceCell::new();
static MYSQL_POOL: OnceCell<HashMap<String, Pool<MySql>>> = OnceCell::new();
static POSTGRES_POOL: OnceCell<HashMap<String, Pool<Postgres>>> = OnceCell::new();

#[async_trait]
pub trait Template<'a, S, DB: Database, A: Arguments<'a>>: Debug where S: Template<'a, S, DB, A> {

    fn get_pool(&self) -> &DbPool {
        let key = format!("{:?}", self);
        POOLS.get().unwrap().get(&key).unwrap()
        // todo!()
    }
/*
    async fn exe<'a, DB: Database, A:  IntoArguments<'a, DB>>(query: Query<'a, DB, A>, pool: &'a DbPool) {
        match pool {
            DbPool::Any(p) => query.execute(p),
            DbPool::Mssql(p) => query.execute(p),
            DbPool::MySql(p) => query.execute(p),
            DbPool::Postgres(p) => query.execute(p),
            DbPool::Sqlite(p) => query.execute(p),
        };
    }
*/
    async fn insert<T: Table>(&self) -> Result<u64, Error> {
        let fields = T::fields_name();
        let args = fields.iter().map(|_| "?").collect::<Vec<&str>>();
        let sql = SqlBuilder::insert_into(T::table_name())
            .fields(fields.as_slice())
            .values(args.as_slice()).sql()?;

        let query = sqlx::query_with("", Args::new().mysql_args());

        // let args = t.bind_args(Args::new());
        // let result = sqlx::query_with(sql.as_str(), args.mysql_args())
        //     .execute(pool())
        //     .await?;
        // Ok(result.rows_affected())
        Ok(0)
    }
    // update ...

    // async fn data_source() -> Result<Vec<(S, Pool<DB>)>, Error>;
    //
    // async fn init() -> Result<(), Error>;
}

#[async_trait]
pub trait MySqlTemplate<'a, S>: Template<'a, S, MySql, MySqlArguments> where S: MySqlTemplate<'a, S> {

    async fn data_source() -> Result<Vec<(S, Pool<MySql>)>, Error>;

    async fn init() -> Result<(), Error> {
        let data_source = Self::data_source().await?;
        let mut a = HashMap::new();
        let b = data_source.first().unwrap().1.clone();
        a.insert("".to_string(), b);
        MYSQL_POOL.set(a);
        todo!()

        // let mut map = HashMap::with_capacity(data_source.len());
        // for x in data_source {
        //     map.insert(format!("{:?}", x.0), x.1);
        // }
        //
        // POOLS.set(map).map_err(|_| anyhow!("Can not init data source."))?;

    }
}


pub enum DbPool {
    Any(Pool<Any>),
    Mssql(Pool<Mssql>),
    MySql(Pool<MySql>),
    Postgres(Pool<Postgres>),
    Sqlite(Pool<Sqlite>)
}

#[derive(Debug)]
pub enum DataSource {
    Default, Other
}

#[async_trait]
impl<'a> Template<'a, DataSource, MySql, MySqlArguments> for DataSource {}

#[async_trait]
impl<'a> MySqlTemplate<'a, DataSource> for DataSource {
    async fn data_source() -> Result<Vec<(DataSource, Pool<MySql>)>, Error> {
        todo!()
    }

    async fn init() -> Result<(), Error> {
        todo!()
    }
}


// #[async_trait]
// impl<'a> MySqlTemplate<'a, DataSource> for DataSource {
//
// }

// let pool = MySqlPoolOptions::new()
// .max_connections(5)
// .connect("config.conn.as_str()")
// .await?;
//
// Ok(vec![(DataSource::Default, pool)])


#[cfg(test)]
mod test {
    #[test]
    fn test() {
    }
}