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
use crate::template::private;

static POOLS: OnceCell<HashMap<String, Pool<MySql>>> = OnceCell::new();

#[async_trait]
pub trait MySqlTemplate<S>: Debug where S: MySqlTemplate<S> {

    async fn insert<T>(&self, t: &T) -> Result<u64, Error> where T: Remap<MySql> + Sync {
        let sql = Self::build_insert_sql::<private::Local, T>(1).sql()?;
        let mut args = MySqlArguments::default();
        t.fields_args().values.iter().for_each(|a| args.add(a));

        let x = sqlx::query_with(sql.as_str(), args)
            .execute(self.pool())
            .await?;

        Ok(x.rows_affected())
    }

    async fn insert_batch<'a, T>(&self, v: &Vec<&T>, tx: Option<&mut Transaction<'a, MySql>>)
        -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        assert!(v.len() > 0);
        let sql = Self::build_insert_sql::<private::Local, T>(v.len()).sql()?;
        let mut arguments = MySqlArguments::default();
        v.iter().for_each(|t| {
            t.fields_args().values.iter().for_each(|x| arguments.add(x));
        });

        let x = self.insert_tx::<private::Local>(sql.as_str(), arguments, tx).await?;
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

    async fn insert_ignore<'a, T>(&self, v: &Vec<&T>, tx: Option<&mut Transaction<'a, MySql>>)
        -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        assert!(v.len() > 0);
        let sql = Self::build_insert_sql::<private::Local, T>(v.len()).sql()?
            .replace("INSERT", "INSERT IGNORE");
        let mut arguments = MySqlArguments::default();
        v.iter().for_each(|t| {
            t.fields_args().values.iter().for_each(|x| arguments.add(x));
        });

        let x = self.insert_tx::<private::Local>(sql.as_str(), arguments, tx).await?;
        Ok(x.rows_affected())
    }

    async fn insert_update<'a, T>(
        &self, v: &Vec<&T>, update_fields: &[&str], tx: Option<&mut Transaction<'a, MySql>>)
        -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        assert!(v.len() > 0 && update_fields.len() > 0);

        let mut sql = Self::build_insert_sql::<private::Local, T>(v.len()).sql()?;
        sql.pop(); // remove ;   sql.remove(sql.len() - 1);   sql.replace(";", "");

        let mut update = String::new();
        update_fields.iter().for_each(|x| {
            if !update.is_empty() { update.push_str(","); }
            let s = format!("{0} = new.{0}", x);
            update.push_str(s.as_str());
        });
        let sql = format!("{} AS new ON DUPLICATE KEY UPDATE {};", sql, update);

        let mut arguments = MySqlArguments::default();
        v.iter().for_each(|t| {
            t.fields_args().values.iter().for_each(|x| arguments.add(x));
        });

        let x = self.insert_tx::<private::Local>(sql.as_str(), arguments, tx).await?;
        Ok(x.rows_affected())
    }

    #[doc(hidden)]
    async fn insert_tx<'a, L: private::IsLocal>(
        &self, sql: &str, arguments: MySqlArguments, tx: Option<&mut Transaction<'a, MySql>>)
        -> Result<MySqlQueryResult, Error> {
        let x = match tx {
            Some(tx) => {
                sqlx::query_with(sql, arguments).execute(tx).await?
            },
            _ => {
                let mut tx = self.pool().begin().await?;
                let x = sqlx::query_with(sql, arguments)
                    .execute(&mut tx)
                    .await?;
                tx.commit().await?;
                x
            }
        };
        Ok(x)
    }

    #[doc(hidden)]
    fn build_insert_sql<L, T>(size: usize) -> SqlBuilder
        where L: private::IsLocal,
              T: Remap<MySql> {
        let mut sql = SqlBuilder::insert_into(T::table_name());
        sql.fields(T::fields_name().as_slice());

        let holders = T::fields_name().iter().map(|_| "?").collect::<Vec<&str>>();
        for _ in 0..size {
            sql.values(holders.as_slice());
        }
        sql
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
    async fn insert() {
        MySqlSource::init().await.unwrap();
        let user = User {id: 2, name: "Sam".into()};
        let a = MySqlSource::Default.insert(&user).await.unwrap();
    }

    #[async_test]
    async fn insert_batch() {
        MySqlSource::init().await.unwrap();
        let user_1 = User {id: 1, name: "Bob".into()};
        let user_2 = User {id: 2, name: "Cat".into()};
        let vec = vec![&user_1, &user_2];

        let rows = MySqlSource::Default.insert_batch(&vec, None).await.unwrap();
        assert_eq!(2, rows);
        let rows = MySqlSource::Default.insert_ignore(&vec, None).await.unwrap();
        assert_eq!(0, rows);

        let user_2 = User {id: 2, name: "Cow".into()};
        let user_3 = User {id: 3, name: "Dog".into()};
        let vec = vec![&user_2, &user_3];
        let rows = MySqlSource::Default.insert_update(&vec, &["name"], None).await.unwrap();
        assert_eq!(3, rows)
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