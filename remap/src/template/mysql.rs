use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Error;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use sql_builder::SqlBuilder;
use sqlx::{Database, MySql, Pool, Transaction, Arguments};

use crate::arguments::Args;
use crate::extend::Remap;
use sqlx::mysql::{MySqlArguments, MySqlQueryResult};
use std::borrow::BorrowMut;

static POOLS: OnceCell<HashMap<String, Pool<MySql>>> = OnceCell::new();

#[async_trait]
pub trait MySqlTemplate<S>: Debug where S: MySqlTemplate<S> {

    async fn insert<T>(&self, t: &T) -> Result<u64, Error> where T : Remap<MySql> + Sync {
        let holders = T::fields_name().iter().map(|_| "?").collect::<Vec<&str>>();
        let sql = SqlBuilder::insert_into(T::table_name())
            .fields(T::fields_name().as_slice())
            .values(holders.as_slice())
            .sql()?;

        let mut args = MySqlArguments::default();
        t.fields_args().values.iter().for_each(|a| args.add(a));

        let x = sqlx::query_with(sql.as_str(), args)
            .execute(self.pool())
            .await?;

        Ok(x.rows_affected())
    }

    async fn insert_batch<'a, T>(&self, v: &Vec<&T>, tx: Option<&mut Transaction<'a, MySql>>)
                                 -> Result<u64, Error>
        where T : Remap<MySql> + Sync {
        let mut sql = SqlBuilder::insert_into(T::table_name());
        sql.fields(T::fields_name().as_slice());

        let holders = T::fields_name().iter().map(|_| "?").collect::<Vec<&str>>();
        let mut arguments = MySqlArguments::default();
        v.iter().for_each(|t| {
            sql.values(holders.as_slice());
            t.fields_args().values.iter().for_each(|x| arguments.add(x));
        });
        let sql = sql.sql()?;

        let x = match tx {
            Some(tx) => {
                sqlx::query_with(sql.as_str(), arguments).execute(tx).await?
            },
            _ => {
                let mut tx = self.pool().begin().await?;
                let x = sqlx::query_with(sql.as_str(), arguments)
                    .execute(&mut tx)
                    .await?;
                tx.commit().await?;
                x
            }
        };

        Ok(x.rows_affected())

        // `Bug and compile failed`
        // let tx = match tx {
        //     Some(tx) => tx,
        //     _ => self.pool().begin().await?.borrow_mut()
        // };
        // let x =  sqlx::query_with(sql.as_str(), args)
        //     .execute(tx)
        //     .await?;
        // tx.commit().await?;
    }

    async fn insert_ignore<'a, T>(v: &Vec<T>, tx: &mut Transaction<'a, MySql>)
        -> Result<u64, Error>
        where T : Remap<MySql> + Sync {
        todo!()
    }

    async fn insert_update<'a, T>(
        v: &Vec<T>, set_fields: &[&str], args: Args<'a, MySql>, tx: &mut Transaction<'a, MySql>)
        -> Result<(), Error>
        where T : Remap<MySql> + Sync {

        todo!()
    }

    async fn update<'a, T>(set_fields: &[&str], and_where_eq: &[&str], args: Args<'a, MySql>)
        -> Result<u64, Error>
        where T : Remap<MySql> + Sync {
        todo!()
    }

    async fn select_one<'a, T>(where_eq: &str, args: Args<'a, MySql>) -> Result<Option<T>, Error>
        where T: Remap<MySql> + Sync {
        todo!()
    }

    async fn select_in<'a, T>(where_in: &str, args: Args<'a, MySql>) -> Result<Vec<T>, Error> {
        todo!()
    }

    fn pool(&self) -> &Pool<MySql> {
        let key = format!("{:?}", self);
        POOLS.get().unwrap().get(&key).unwrap()
    }

    async fn data_source() -> Result<Vec<(S, Pool<MySql>)>, Error>;

    async fn init() -> Result<(), Error> {
        let data_source = Self::data_source().await?;
        let mut map = HashMap::with_capacity(data_source.len());
        for (key, value) in data_source {
            map.insert(format!("{:?}", key), value);
        }
        POOLS.set(map).map_err(|_| anyhow!("Can not init data source."))?;
        Ok(())
    }
}

#[cfg(test)]
mod example {
    use anyhow::Error;
    use async_trait::async_trait;
    use futures_await_test::async_test;
    use sqlx::{MySql, Pool};
    use sqlx::mysql::MySqlPoolOptions;

    use crate as remap;
    // use crate::extend::Remap;
    use crate::template::mysql::MySqlTemplate;

    #[async_test]
    async fn abc() {
        MySqlSource::init().await.unwrap();

        let user = User {id: 2, name: "小猪".into()};
        let a = MySqlSource::Default.insert(&user).await.unwrap();
    }

    #[derive(Debug, Remap)]
    #[remap(sqlx::MySql, table = "user")]
    pub struct User {
        id: u32,
        name: String
    }

    #[derive(Debug)]
    enum MySqlSource {
        Default, Other
    }

    #[async_trait]
    impl MySqlTemplate<MySqlSource> for MySqlSource {
        async fn data_source() -> Result<Vec<(MySqlSource, Pool<MySql>)>, Error> {
            let pool = MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:123456@localhost:3306/remap")
                .await?;
            let pool2 = MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:123456@localhost:3306/remap")
                .await?;

            // Ok(vec![(MySqlSource::Default, pool), (MySqlSource::Other, pool2)])
            Ok(vec![(MySqlSource::Default, pool)])
        }

    }
}