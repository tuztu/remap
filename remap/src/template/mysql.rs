use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Error;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use sql_builder::SqlBuilder;
use sqlx::{Arguments, Database, FromRow, MySql, Pool, Transaction};
use sqlx::mysql::{MySqlArguments, MySqlQueryResult, MySqlRow};

use crate::arguments::Args;
use crate::extend::Remap;
use crate::template::private;

static POOLS: OnceCell<HashMap<String, Pool<MySql>>> = OnceCell::new();

#[async_trait]
pub trait MySqlTemplate<S>: Debug where S: MySqlTemplate<S> {

    async fn insert_one<T>(&self, t: &T) -> Result<u64, Error> where T: Remap<MySql> + Sync {
        let sql = Self::build_insert_sql::<private::Local, T>(1).sql()?;
        let mut args = MySqlArguments::default();
        t.fields_args().values.iter().for_each(|a| args.add(a));

        let x = sqlx::query_with(sql.as_str(), args)
            .execute(self.pool())
            .await?;

        Ok(x.rows_affected())
    }

    async fn insert<'a, T>(&self, v: &[T], tx: Option<&mut Transaction<'a, MySql>>)
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

    async fn insert_ignore<'a, T>(&self, v: &[T], tx: Option<&mut Transaction<'a, MySql>>)
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

    // todo insert_replace

    async fn insert_update<'a, T>(
        &self, v: &[T], fields: &[&str], tx: Option<&mut Transaction<'a, MySql>>)
        -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        assert!(v.len() > 0 && fields.len() > 0);

        let mut sql = Self::build_insert_sql::<private::Local, T>(v.len()).sql()?;
        sql.pop(); // remove ;   sql.remove(sql.len() - 1);   sql.replace(";", "");

        let mut update = String::new();
        fields.iter().for_each(|x| {
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
            }
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

    async fn select_one<'a, T>(&self, field_eq: &str, args: &'a Args<'a, MySql>)
        -> Result<Option<T>, Error>
        where T: Remap<MySql> + Sync {
        assert!(!args.values.is_empty());
        let sql = SqlBuilder::select_from(T::table_name())
            .and_where_eq(field_eq, "?")
            .sql()?;

        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x));

        let x = sqlx::query_with(sql.as_str(), arguments)
            .map(|row| T::decode_row(row))
            .fetch_optional(self.pool())
            .await?;

        match x {
            Some(s) => Ok(Some(s?)),
            _ => Ok(None)
        }
    }

    async fn select_in<'a, T>(&self, field_in: &str, args: &'a Args<'a, MySql>)
        -> Result<Vec<T>, Error>
        where T: Remap<MySql> + Sync {
        assert!(!args.values.is_empty());
        let mut holders = Vec::with_capacity(args.values.len());
        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| {
            holders.push("?");
            arguments.add(x)
        });

        let sql = SqlBuilder::select_from(T::table_name())
            .and_where_in(field_in, holders.as_slice())
            .sql()?;
        let output = sqlx::query_with(sql.as_str(), arguments)
            // .map(|row| T::decode_row(row))
            .fetch_all(self.pool())
            .await?;
        let mut vec = Vec::with_capacity(output.len());
        for row in output {
            vec.push(T::decode_row(row)?);
        }
        Ok(vec)
    }

    async fn select_as<'a, T>(&self, sql: &str, args: &'a Args<'a, MySql>)
        -> Result<Vec<T>, Error>
        where T: for<'r> FromRow<'r, MySqlRow> + Send + Unpin {
        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x) );
        let output = sqlx::query_as_with(sql, arguments)
            .fetch_all(self.pool())
            .await?;
        Ok(output)
    }

    async fn select<'a, T>(&self, sql: &str, args: &'a Args<'a, MySql>)
        -> Result<Vec<T>, Error>
        where T: Remap<MySql> + Sync {
        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x) );
        let output = sqlx::query_with(sql, arguments)
            .fetch_all(self.pool())
            .await?;
        let mut vec = Vec::with_capacity(output.len());
        for row in output {
            vec.push(T::decode_row(row)?);
        }
        Ok(vec)
    }

    async fn update<'a, T>(&self, set_fields: &[&str], fields_eq: &[&str], args: &'a Args<'a, MySql>)
       -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        let mut sql = SqlBuilder::update_table(T::table_name());
        set_fields.iter().for_each(|x| { sql.set(x, "?"); });
        fields_eq.iter().for_each(|x| { sql.and_where_eq(x, "?"); });
        let sql = sql.sql()?;

        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x));
        let x = sqlx::query_with(sql.as_str(), arguments)
            .execute(self.pool())
            .await?;
        Ok(x.last_insert_id())
    }

    async fn delete<'a, T>(&self, fields_eq: &[&str], args: &'a Args<'a, MySql>)
        -> Result<u64, Error>
        where T: Remap<MySql> + Sync {
        let mut sql = SqlBuilder::delete_from(T::table_name());
        fields_eq.iter().for_each(|x| { sql.and_where_eq(x, "?"); });
        let sql = sql.sql()?;

        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x));
        let x = sqlx::query_with(sql.as_str(), arguments)
            .execute(self.pool())
            .await?;
        Ok(x.last_insert_id())
    }

    async fn execute<'a>(&self, sql: &str, args: &'a Args<'a, MySql>) -> Result<u64, Error> {
        let mut arguments = MySqlArguments::default();
        args.values.iter().for_each(|x| arguments.add(x) );
        let output = sqlx::query_with(sql, arguments)
            .execute(self.pool())
            .await?;
        Ok(output.rows_affected())
    }

    fn pool(&self) -> &Pool<MySql> {
        POOLS.get().unwrap().get(&format!("{:?}", self)).unwrap()
    }

    async fn data_source() -> Result<Vec<(S, Pool<MySql>)>, Error>;

    async fn init() -> Result<(), Error> {
        if POOLS.get().is_some() { return Ok(()); }
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
        let user = User { id: 2, name: "Sam".into() };
        let a = MySqlSource::Default.insert_one(&user).await.unwrap();
    }

    #[async_test]
    async fn insert_batch() {
        MySqlSource::init().await.unwrap();
        let user_1 = User { id: 1, name: "Bob".into() };
        let user_2 = User { id: 2, name: "Cat".into() };
        let vec = [user_1, user_2];

        let rows = MySqlSource::Default.insert(&vec, None).await.unwrap();
        assert_eq!(2, rows);
        let rows = MySqlSource::Default.insert_ignore(&vec, None).await.unwrap();
        assert_eq!(0, rows);

        let user_2 = User { id: 2, name: "Cow".into() };
        let user_3 = User { id: 3, name: "Dog".into() };
        let vec = [user_2, user_3];
        let rows = MySqlSource::Default.insert_update(&vec, &["name"], None).await.unwrap();
        assert_eq!(3, rows)
    }


    #[derive(Debug, Remap)]
    #[remap(MySql, table = "user")]
    pub struct User {
        id: u32,
        name: String,
    }

    #[derive(Debug)]
    enum MySqlSource {
        Default,
        Other,
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