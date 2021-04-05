use sqlx::{Any, Database, Encode, Mssql, MySql, Postgres, Sqlite, Type, TypeInfo};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;

/*
pub trait Arguments<'a, DB>: Sized where DB: Database {
    fn new() -> Self;
    fn from<T>(v: T)  -> Self where T: 'a + Send + Encode<'a, DB> + Type<DB>;
    fn add<T>(self, v: T) -> Self where T: 'a + Send + Encode<'a, DB> + Type<DB>;
    // fn append(&mut self, arg: &mut A);
    fn append(&mut self, arg: &mut Self);
    fn cast<T>(v: T) -> Bridge<'a, DB>
        where T: 'a + Send + Encode<'a, DB> + Type<DB>,
              <DB as HasArguments<'a>>::ArgumentBuffer: Default {
        let ty = v.produces().unwrap_or_else(T::type_info);
        let compatible = T::compatible(&ty);
        let size_hint = v.size_hint();
        let mut buffer = Default::default();
        let is_null = v.encode(&mut buffer);

        Bridge {
            value: buffer,
            is_null,
            size_hint,
            ty,
            compatible
        }
    }
}

impl<'a> Arguments<'a, MySql> for Args<'a, MySql> {
    fn new() -> Self {
        Args { values: vec![] }
    }

    fn from<T>(v: T) -> Self where T: 'a + Send + Encode<'a, MySql> + Type<MySql> {
        Args { values: vec![Self::cast(v)] }
    }

    fn add<T>(mut self, v: T) -> Self where T: 'a + Send + Encode<'a, MySql> + Type<MySql> {
        self.values.push(Self::cast(v));
        self
    }

    fn append(&mut self, arg: &mut Self) { // arg: &mut MySqlArgs<'a>
        self.values.append(&mut arg.values);
    }
}
*/

pub struct Args<'a, DB: Database> {
    pub values: Vec<Bridge<'a, DB>>
}

impl<'a, DB: Database> Args<'a, DB> {

    pub fn new() -> Self {
        Args { values: vec![] }
    }

    pub fn from<T>(v: T) -> Self
        where T: 'a + Send + Encode<'a, DB> + Type<DB>,
              <DB as HasArguments<'a>>::ArgumentBuffer: Default {
        Args { values: vec![Self::cast(v)] }
    }

    pub fn add<T>(mut self, v: T) -> Self
        where T: 'a + Send + Encode<'a, DB> + Type<DB>,
              <DB as HasArguments<'a>>::ArgumentBuffer: Default {
        self.values.push(Self::cast(v));
        self
    }
    pub fn append(&mut self, arg: &mut Self) { // arg: &mut MySqlArgs<'a>
        self.values.append(&mut arg.values);
    }

    fn cast<T>(v: T) -> Bridge<'a, DB>
        where T: 'a + Send + Encode<'a, DB> + Type<DB>,
              <DB as HasArguments<'a>>::ArgumentBuffer: Default {
        let ty = v.produces().unwrap_or_else(T::type_info);
        let compatible = T::compatible(&ty);
        let size_hint = v.size_hint();
        let mut buffer = Default::default();
        let is_null = v.encode(&mut buffer);

        Bridge {
            value: buffer,
            is_null,
            size_hint,
            ty,
            compatible
        }
    }
}

pub struct Bridge<'a, DB: Database> {
    value: <DB as HasArguments<'a>>::ArgumentBuffer,
    is_null: IsNull,
    size_hint: usize,

    ty: <DB as Database>::TypeInfo,
    compatible: bool
}

impl<'a> Encode<'a, MySql> for Bridge<'a, MySql> {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'a>>::ArgumentBuffer) -> IsNull {
        <&[u8] as Encode<MySql>>::encode(self.value.as_slice(), buf);
        match self.is_null {
            IsNull::Yes => IsNull::Yes,
            IsNull::No => IsNull::No
        }
    }

    fn produces(&self) -> Option<<MySql as Database>::TypeInfo> {
        Some(self.ty.clone())
    }

    fn size_hint(&self) -> usize {
        self.size_hint
    }
}


impl<'a> Type<MySql> for Bridge<'a, MySql> {
    /// this function for add arguments function get type,
    /// `Bridge` store the type info and return by produces(&self) function,
    /// the function would never call.
    fn type_info() -> <MySql as Database>::TypeInfo {
        panic!("This wasn’t supposed to happen");
        todo!()
    }

    /// This function for decoding values from a row and query macro.
    /// Never call through `Bridge` type.
    fn compatible(ty: &<MySql as Database>::TypeInfo) -> bool {
        panic!("This wasn’t supposed to happen");
        todo!()
    }
}


impl<'a> Encode<'a, Postgres> for Bridge<'a, Postgres> {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'_>>::ArgumentBuffer) -> IsNull {
        buf.push(0);
        todo!()
    }
}
