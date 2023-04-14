#[cfg(feature = "postgres")] use canyon_connection::tokio_postgres;
#[cfg(feature = "mssql")] use canyon_connection::tiberius;

use crate::crud::Transaction;

/// Declares functions that takes care to deserialize data incoming
/// from some supported database in Canyon-SQL into a user's defined
/// type `T`
pub trait RowMapper<T: Transaction<T>>: Sized {
    fn deserialize_postgresql(row: &tokio_postgres::Row) -> T;

    fn deserialize_sqlserver(row: &tiberius::Row) -> T;
}
