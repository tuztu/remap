use anyhow::Error;
use once_cell::sync::OnceCell;
use sqlx::{Arguments, Encode, MySql, Pool, Type};
use sqlx::mysql::{MySqlArguments, MySqlRow};

static POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub async fn init_pool(pool: Pool<MySql>) -> Result<(), Error> {
    // let pool = MySqlPoolOptions::new()
    //     .max_connections(5)
    //     .connect(config.conn.as_str())
    //     .await?;
    POOL.set(pool).map_err(|_| anyhow!("Failed to setup mysql pools."))
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

    pub(crate) fn args_size(&self) -> usize {
        self.args_size
    }

    pub(crate) fn mysql_args(self) -> MySqlArguments {
        self.mysql_args
    }
}

/*
use sqlx::encode::{IsNull};
use sqlx::mysql::{MySqlTypeInfo};

/// Implementation of [`Arguments`] for MySQL.
#[derive(Debug, Default)]
pub struct MySqlArgument {
    pub(crate) values: Vec<u8>,
    pub(crate) types: Vec<MySqlTypeInfo>,
    pub(crate) null_bitmap: Vec<u8>,
}

impl MySqlArgument {
    pub(crate) fn add<'q, T>(&mut self, value: T) where T: Encode<'q, MySql> + Type<MySql> {
        let index = self.types.len();
        self.null_bitmap.resize((index / 8) + 1, 0);

        let ty = value.produces().unwrap_or_else(T::type_info);
        self.types.push(ty);

        if let IsNull::Yes = value.encode(&mut self.values) {
            self.null_bitmap[index / 8] |= (1 << (index % 8)) as u8;
        }
    }
}
*/