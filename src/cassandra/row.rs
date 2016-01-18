use cassandra_sys::CassIterator as _CassIterator;
use cassandra_sys::cass_iterator_free;
use cassandra_sys::cass_iterator_next;
use cassandra_sys::cass_iterator_get_column;
use cassandra_sys::CassRow as _Row;
use cassandra_sys::cass_row_get_column;
use cassandra_sys::cass_row_get_column_by_name;
use cassandra_sys::cass_iterator_from_row;
use cassandra_sys::CASS_ERROR_LIB_INDEX_OUT_OF_BOUNDS;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;
use std::ffi::CString;
use std::iter::IntoIterator;
use std::iter;

use cassandra::error::CassError;
use cassandra::column::Column;
use cassandra::column;

///A collection of column values.
pub struct Row(*const _Row);

pub mod protected {
    use cassandra::row::Row;
    use cassandra_sys::CassRow as _Row;
    pub fn build(row: *const _Row) -> Row {
        Row(row)
    }
}

use cassandra::value;
impl Debug for Row {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for column in self {
            try!(write!(f,
                        "{:?}\t",
                        value::protected::build(column::protected::inner(&column))));
        }
        Ok(())
    }
}

impl Display for Row {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for column in self {
            try!(write!(f,
                        "{}\t",
                        value::protected::build(column::protected::inner(&column))));
        }
        Ok(())
    }
}

impl Row {
    ///Get a particular column by index
    pub fn get_column(&self, index: u64) -> Result<Column, CassError> {
        unsafe {
            let col = cass_row_get_column(self.0, index);
            if col.is_null() {
                Err(CassError::build(CASS_ERROR_LIB_INDEX_OUT_OF_BOUNDS, None))
            } else {
                Ok(column::protected::build(col))
            }
        }
    }

    ///Get a particular column by name
    pub fn get_column_by_name<S>(&self, name: S) -> Column
        where S: Into<String> {
        unsafe {
            let name = CString::new(name.into()).unwrap();
            println!("name: {:?}", name);
            println!("self: {:?}", self);
            // unimplemented!();
            column::protected::build(cass_row_get_column_by_name(self.0, name.as_ptr()))
        }
    }
}

///An iterator over the columns in a row
pub struct RowIterator(pub *mut _CassIterator);


impl Drop for RowIterator {
    fn drop(&mut self) {
        unsafe { cass_iterator_free(self.0) }
    }
}

impl iter::Iterator for RowIterator {
    type Item = Column;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            match cass_iterator_next(self.0) {
                0 => None,
                _ => Some(column::protected::build(cass_iterator_get_column(self.0))),
            }
        }
    }
}

impl<'a> Iterator for &'a RowIterator {
    type Item = Column;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            match cass_iterator_next(self.0) {
                0 => None,
                _ => Some(column::protected::build(cass_iterator_get_column(self.0))),
            }
        }
    }
}

impl Display for RowIterator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for item in self {
            try!(write!(f,
                        "{}\t",
                        value::protected::build(column::protected::inner(&item))));
        }
        Ok(())
    }
}

impl IntoIterator for Row {
    type Item = Column;
    type IntoIter = RowIterator;

    ///Creates a new iterator for the specified row. This can be
    ///used to iterate over columns in a row.
    fn into_iter(self) -> Self::IntoIter {
        unsafe { RowIterator(cass_iterator_from_row(self.0)) }
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = Column;
    type IntoIter = RowIterator;
    fn into_iter(self) -> Self::IntoIter {
        unsafe { RowIterator(cass_iterator_from_row(self.0)) }
    }
}