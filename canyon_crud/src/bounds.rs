use crate::{
    crud::{CrudOperations, Transaction},
    mapper::RowMapper,
};

#[cfg(feature = "tokio-postgres")]
use canyon_connection::tokio_postgres::{self, types::ToSql};

#[cfg(feature = "tiberius")]
use canyon_connection::tiberius::{self, ColumnData, FromSql, IntoSql};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::any::Any;

/// Created for retrieve the field's name of a field of a struct, giving
/// the Canyon's autogenerated enum with the variants that maps this
/// fields.
///
/// ```
/// pub struct Struct<'a> {
///     pub some_field: &'a str
/// }
///
/// // Autogenerated enum
/// #[derive(Debug)]
/// #[allow(non_camel_case_types)]
/// pub enum StructField {
///     some_field
/// }
/// ```
/// So, to retrieve the field's name, something like this w'd be used on some part
/// of the Canyon's Manager crate, to wire the necessary code to pass the field
/// name, retrieved from the enum variant, to a called.
///
/// // Something like:
/// `let struct_field_name_from_variant = StructField::some_field.field_name_as_str();`
pub trait FieldIdentifier<T>
where
    T: Transaction<T> + CrudOperations<T> + RowMapper<T>,
{
    fn as_str(&self) -> &'static str;
}

/// Represents some kind of introspection to make the implementors
/// able to retrieve a value inside some variant of an associated enum type.
/// and convert it to a tuple struct formed by the column name as an String,
/// and the dynamic value of the [`QueryParameter<'_>`] trait object contained
/// inside the variant requested,
/// enabling a conversion of that value into something
/// that can be part of an SQL query.
///
///
/// Ex:
/// `SELECT * FROM some_table WHERE id = 2`
///
/// That '2' it's extracted from some enum that implements [`FieldValueIdentifier`],
/// where usually the variant w'd be something like:
///
/// ```
/// pub enum Enum {
///     IntVariant(i32)
/// }
/// ```
pub trait FieldValueIdentifier<'a, T>
where
    T: Transaction<T> + CrudOperations<T> + RowMapper<T>,
{
    fn value(self) -> (&'static str, &'a dyn QueryParameter<'a>);
}

/// Bounds to some type T in order to make it callable over some fn parameter T
///
/// Represents the ability of an struct to be considered as candidate to perform
/// actions over it as it holds the 'parent' side of a foreign key relation.
///
/// Usually, it's used on the Canyon macros to retrieve the column that
/// this side of the relation it's representing
pub trait ForeignKeyable<T> {
    /// Retrieves the field related to the column passed in
    fn get_fk_column(&self, column: &str) -> Option<&dyn QueryParameter<'_>>;
}

/// Generic abstraction to represent any of the Row types
/// from the client crates
pub trait Row {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "tokio-postgres")] impl Row for tokio_postgres::Row {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "tiberius")] impl Row for tiberius::Row {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Generic abstraction for hold a Column type that will be one of the Column
/// types present in the dependent crates
// #[derive(Copy, Clone)]
pub struct Column<'a> {
    name: &'a str,
    type_: ColumnType,
}
impl<'a> Column<'a> {
    pub fn name(&self) -> &'_ str {
        self.name
    }
    pub fn column_type(&self) -> &ColumnType {
        &self.type_
    }
    // pub fn type_(&'a self) -> &'_ dyn Type {
    //     match (*self).type_ {
    //         #[cfg(feature = "tokio-postgres")] ColumnType::Postgres(v) => v as &'a dyn Type,
    //         #[cfg(feature = "tiberius")] ColumnType::SqlServer(v) => v as &'a dyn Type,
    //     }
    // }
}

pub trait Type {
    fn as_any(&self) -> &dyn Any;
}
#[cfg(feature = "tokio-postgres")] impl Type for tokio_postgres::types::Type {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[cfg(feature = "tiberius")] impl Type for tiberius::ColumnType {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Wrapper over the dependencies Column's types
// #[derive(Copy)]
pub enum ColumnType {
    #[cfg(feature = "tokio-postgres")] Postgres(tokio_postgres::types::Type),
    #[cfg(feature = "tiberius")] SqlServer(tiberius::ColumnType),
}

pub trait RowOperations {
    #[cfg(feature = "tokio-postgres")]
    fn get_postgres<'a, Output>(&'a self, col_name: &str) -> Output
        where Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "tiberius")]
    fn get_mssql<'a, Output>(&self, col_name: &str) -> Output
        where Output: tiberius::FromSql<'a>;

    #[cfg(feature = "tokio-postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &str) -> Option<Output>
        where Output: tokio_postgres::types::FromSql<'a>;
    #[cfg(feature = "tiberius")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &str) -> Option<Output>
        where Output: tokio_postgres::types::FromSql<'a>;

    fn columns(&self) -> Vec<Column>;
}

impl RowOperations for &dyn Row {
    #[cfg(feature = "tokio-postgres")]
    fn get_postgres<'a, Output>(&'a self, col_name: &str) -> Output
        where Output: tokio_postgres::types::FromSql<'a>
    {
        if let Some(row) = self.as_any().downcast_ref::<tokio_postgres::Row>() {
            return row.get::<&str, Output>(col_name);
        };
        panic!() // TODO into result and propagate
    }
    #[cfg(feature = "tiberius")]
    fn get_mssql<'a, Output>(&'a self, col_name: &str) -> Output
        where Output: tiberius::FromSql<'a>
    {
        if let Some(row) = self.as_any().downcast_ref::<tiberius::Row>() {
            return row
                .get::<Output, &str>(col_name)
                .expect("Failed to obtain a row in the MSSQL migrations");
        };
        panic!() // TODO into result and propagate
    }

    #[cfg(feature = "tokio-postgres")]
    fn get_postgres_opt<'a, Output>(&'a self, col_name: &str) -> Option<Output>
        where Output: tokio_postgres::types::FromSql<'a>
    {
        if let Some(row) = self.as_any().downcast_ref::<tokio_postgres::Row>() {
            return row.get::<&str, Option<Output>>(col_name);
        };
        panic!() // TODO into result and propagate
    }

    #[cfg(feature = "tiberius")]
    fn get_mssql_opt<'a, Output>(&'a self, col_name: &str) -> Option<Output>
        where Output: tiberius::FromSql<'a>
    {
        if let Some(row) = self.as_any().downcast_ref::<tiberius::Row>() {
            return row
                .try_get
                .expect("Failed to obtain a row for MSSQL");
        };
        panic!() // TODO into result and propagate
    }

    fn columns(&self) -> Vec<Column> {
        let mut cols = vec![];

        /* if self.as_any().is::<tokio_postgres::Row>() {
            self.as_any()
                .downcast_ref::<tokio_postgres::Row>()
                .expect("Not a tokio postgres Row for column")
                .columns()
                .iter()
                .for_each(|c| {
                    cols.push(Column {
                        name: c.name(),
                        type_: ColumnType::Postgres(c.type_().to_owned()),
                    })
                })
        } else {
            self.as_any()
                .downcast_ref::<tiberius::Row>()
                .expect("Not a Tiberius Row for column")
                .columns()
                .iter()
                .for_each(|c| {
                    cols.push(Column {
                        name: c.name(),
                        type_: ColumnType::SqlServer(c.column_type()),
                    })
                })
        }; */

        cols
    }
}

/// Defines a trait for represent type bounds against the allowed
/// data types supported by Canyon to be used as query parameters.
pub trait QueryParameter<'a>: std::fmt::Debug + Sync + Send {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync);
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_>;
}

/// The implementation of the [`canyon_connection::tiberius`] [`IntoSql`] for the
/// query parameters.
///
/// This implementation is necessary because of the generic amplitude
/// of the arguments of the [`Transaction::query`], that should work with
/// a collection of [`QueryParameter<'a>`], in order to allow a workflow
/// that is not dependent of the specific type of the argument that holds
/// the query parameters of the database connectors
#[cfg(feature = "tiberius")]
impl<'a> IntoSql<'a> for &'a dyn QueryParameter<'a> {
    fn into_sql(self) -> ColumnData<'a> {
        self.as_sqlserver_param()
    }
}

impl<'a> QueryParameter<'a> for bool {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::Bit(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for i16 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for &i16 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(**self))
    }
}
impl<'a> QueryParameter<'a> for Option<i16> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(*self)
    }
}
impl<'a> QueryParameter<'a> for Option<&i16> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I16(Some(*self.unwrap()))
    }
}
impl<'a> QueryParameter<'a> for i32 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for &i32 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(**self))
    }
}
impl<'a> QueryParameter<'a> for Option<i32> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(*self)
    }
}
impl<'a> QueryParameter<'a> for Option<&i32> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I32(Some(*self.unwrap()))
    }
}
impl<'a> QueryParameter<'a> for f32 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for &f32 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(**self))
    }
}
impl<'a> QueryParameter<'a> for Option<f32> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(*self)
    }
}
impl<'a> QueryParameter<'a> for Option<&f32> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")]  fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F32(Some(
            *self.expect("Error on an f32 value on QueryParameter<'_>"),
        ))
    }
}
impl<'a> QueryParameter<'a> for f64 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for &f64 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(**self))
    }
}
impl<'a> QueryParameter<'a> for Option<f64> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(*self)
    }
}
impl<'a> QueryParameter<'a> for Option<&f64> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::F64(Some(
            *self.expect("Error on an f64 value on QueryParameter<'_>"),
        ))
    }
}
impl<'a> QueryParameter<'a> for i64 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(*self))
    }
}
impl<'a> QueryParameter<'a> for &i64 {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(**self))
    }
}
impl<'a> QueryParameter<'a> for Option<i64> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(*self)
    }
}
impl<'a> QueryParameter<'a> for Option<&i64> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(*self.unwrap()))
    }
}
impl<'a> QueryParameter<'a> for String {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Owned(self.to_owned())))
    }
}
impl<'a> QueryParameter<'a> for &String {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Borrowed(self)))
    }
}
impl<'a> QueryParameter<'a> for Option<String> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match self {
            Some(string) => ColumnData::String(Some(std::borrow::Cow::Owned(string.to_owned()))),
            None => ColumnData::String(None),
        }
    }
}
impl<'a> QueryParameter<'a> for Option<&String> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")]  fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match self {
            Some(string) => ColumnData::String(Some(std::borrow::Cow::Borrowed(string))),
            None => ColumnData::String(None),
        }
    }
}
impl<'a> QueryParameter<'_> for &'_ str {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        ColumnData::String(Some(std::borrow::Cow::Borrowed(*self)))
    }
}
impl<'a> QueryParameter<'a> for Option<&'_ str> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        match *self {
            Some(str) => ColumnData::String(Some(std::borrow::Cow::Borrowed(str))),
            None => ColumnData::String(None),
        }
    }
}
impl<'a> QueryParameter<'_> for NaiveDate {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'a> for Option<NaiveDate> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'_> for NaiveTime {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'a> for Option<NaiveTime> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'_> for NaiveDateTime {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")]  fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'a> for Option<NaiveDateTime> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")]  fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'_> for DateTime<FixedOffset> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")]  fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'a> for Option<DateTime<FixedOffset>> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'_> for DateTime<Utc> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
impl<'a> QueryParameter<'_> for Option<DateTime<Utc>> {
    #[cfg(feature = "tokio-postgres")] fn as_postgres_param(&self) -> &(dyn ToSql + Sync) {
        self
    }
    #[cfg(feature = "tiberius")] fn as_sqlserver_param(&self) -> ColumnData<'_> {
        self.into_sql()
    }
}
