use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Error;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool, Transaction, Database};
use crate::config::{Table, Args};

static POOLS: OnceCell<HashMap<String, Pool<MySql>>> = OnceCell::new();

#[async_trait]
pub trait MySqlTemplate<S>: Debug where S: MySqlTemplate<S> {

    async fn insert<T, B: Database>(&self, t: &T) -> Result<u64, Error> where T : Table + Sync {
        todo!()
    }

    async fn insert_tx<'a, T>(v: &Vec<T>, tx: &mut Transaction<'a, MySql>)
        -> Result<u64, Error>
        where T : Table + Sync {


        todo!()
    }

    async fn insert_ignore_tx<'a, T>(v: &Vec<T>, tx: &mut Transaction<'a, MySql>)
        -> Result<u64, Error>
        where T : Table + Sync {
        todo!()
    }

    async fn insert_update_tx<'a, T>(
        v: &Vec<T>, set_fields: &[&str], args: Args, tx: &mut Transaction<'a, MySql>)
        -> Result<(),Error>
        where T : Table + Sync {

        todo!()
    }

    async fn update<T>(set_fields: &[&str], and_where_eq: &[&str], args: Args)
        -> Result<u64, Error>
        where T : Table + Sync {
        todo!()
    }

    async fn select_one<T>(where_eq: &str, args: Args) -> Result<Option<T>, Error>
        where T: Table + Sync {
        todo!()
    }

    async fn select_in<T>(where_in: &str, args: Args) -> Result<Vec<T>, Error> {
        todo!()
    }


    async fn data_source() -> Result<Vec<(S, Pool<MySql>)>, Error>;

    async fn init() -> Result<(), Error> {
        let data_source = Self::data_source().await?;
        let mut map = HashMap::with_capacity(data_source.len());
        for x in data_source {
            map.insert(format!("{:?}", x.0), x.1);
        }
        POOLS.set(map).map_err(|_| anyhow!("Can not init data source."))?;
        Ok(())
    }
}

mod example {
    use anyhow::Error;
    use async_trait::async_trait;
    use sqlx::{MySql, Pool};
    use sqlx::mysql::MySqlPoolOptions;

    use crate::template::mysql::MySqlTemplate;

    fn abc() {
        MySqlSource::init();
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
                .connect("config.conn.as_str()")
                .await?;

            Ok(vec![(MySqlSource::Default, pool)])
        }
    }



}