pub mod mysql;
pub mod postgres;
pub mod sqlite;
pub mod mssql;

// For public trait hide function.
pub(crate) mod private {
    // Once again, we introduce an effectively
    // private type to encode locality.
    // This time, we make it uninhabited so we
    // *cannot* accidentally leak it.
    pub enum Local {}

    // However, we pair it with a 'sealed' trait
    // that is *only* implemented for `Local`.
    pub trait IsLocal {}

    impl IsLocal for Local {}
}