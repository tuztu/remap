use anyhow::Error;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool, Arguments, Encode, Type};
use sqlx::mysql::{MySqlArguments, MySqlRow, MySqlPoolOptions};

static POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub async fn setup(pool: Pool<MySql>) -> Result<(), Error> {
    // let pool = MySqlPoolOptions::new()
    //     .max_connections(5)
    //     .connect(config.conn.as_str())
    //     .await?;
    POOL.set(pool).map_err(|_| anyhow!("Setup mysql with error."))
}

pub fn pool<'a>() -> &'a Pool<MySql> {
    POOL.get().unwrap()
}


pub trait Table: Sized + Send + Unpin {
    fn struct_name() -> String;
    fn fields_name() -> Vec<String>;
    fn bind_args(&self, args: Args) -> Args;
    fn from_mysql_row(row: MySqlRow) -> Result<Self, Error>;
}

pub struct Args {
    mysql_args: MySqlArguments,
    args_size: usize
}

impl Args {
    pub fn new() -> Self {
        Args { mysql_args: MySqlArguments::default(), args_size: 0 }
    }

    pub fn from<'a, T>(v: T) -> Self where T: 'a + Send + Encode<'a, MySql> + Type<MySql> {
        Self::new().bind(v)
    }

    pub fn bind<'a, T>(mut self, v: T) -> Self where T: 'a + Send + Encode<'a, MySql> + Type<MySql> {
        self.mysql_args.add(v);
        self.args_size += 1;
        self
    }

    pub fn args_size(&self) -> usize {
        self.args_size
    }

    pub fn mysql_args(self) -> MySqlArguments {
        self.mysql_args
    }
}