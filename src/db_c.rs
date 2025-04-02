use crate::db::{DB, QueryResult};
use crate::cc::{cstr_to_rust, rust_to_cstr, ngenrs_free_ptr, box_into_raw_new};
use std::ffi::{c_char, CStr};
use std::ptr;

#[repr(C)]
pub struct NGenRSDB {
    db: DB,
}

#[repr(C)]
pub struct NGenRSDBQueryResult {
    result: QueryResult<'static>,
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_open(path: *const c_char) -> *mut NGenRSDB {
    if path.is_null() {
        return ptr::null_mut();
    }

    let path_str = match { cstr_to_rust(path) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match DB::open(&path_str) {
        Ok(db) => { box_into_raw_new(NGenRSDB { db }) },
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_exec(db: *mut NGenRSDB, sql: *const c_char) -> bool {
    if db.is_null() || sql.is_null() {
        return false;
    }

    let db = unsafe { &*db };
    let sql_str = match { cstr_to_rust(sql) } {
        Some(s) => s,
        None => return false,
    };

    db.db.exec(&sql_str).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_query(db: *mut NGenRSDB, sql: *const c_char) -> *mut NGenRSDBQueryResult {
    if db.is_null() || sql.is_null() {
        return ptr::null_mut();
    }

    let db = unsafe { &mut *db };
    let sql_str = match { cstr_to_rust(sql) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match db.db.query(&sql_str) {
        Ok(result) => {
            let result = unsafe { std::mem::transmute(result) };
            box_into_raw_new(NGenRSDBQueryResult { result })
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_string(result: *mut NGenRSDBQueryResult, column: *const c_char) -> *mut c_char {
    if result.is_null() || column.is_null() {
        return ptr::null_mut();
    }

    let result = unsafe { &mut *result };
    let column_str = match { cstr_to_rust(column) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match result.result.next_row().and_then(|row| row.get_string(&column_str)) {
        Some(s) => { rust_to_cstr(s) },
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_i64(result: *mut NGenRSDBQueryResult, column: *const c_char) -> i64 {
    if result.is_null() || column.is_null() {
        return 0;
    }

    let result = unsafe { &mut *result };
    let c_str = unsafe { CStr::from_ptr(column) };
    let column_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    result.result.next_row()
        .and_then(|row| row.get_i64(column_str))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_f64(result: *mut NGenRSDBQueryResult, column: *const c_char) -> f64 {
    if result.is_null() || column.is_null() {
        return 0.0;
    }

    let result = unsafe { &mut *result };
    let c_str = unsafe { CStr::from_ptr(column) };
    let column_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0.0,
    };

    result.result.next_row()
        .and_then(|row| row.get_f64(column_str))
        .unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_string(s: *mut c_char) {
    ngenrs_free_ptr(s)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_database(db: *mut NGenRSDB) {
    ngenrs_free_ptr(db)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_result(result: *mut NGenRSDBQueryResult) {
    ngenrs_free_ptr(result)
}