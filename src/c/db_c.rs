use crate::core::db::{DB, QueryResult};
use crate::c::util::{cstr_to_rust, rust_to_cstr, ngenrs_free_ptr, box_into_raw_new};
use std::ffi::{c_void, c_char};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_open(path: *const c_char) -> *mut c_void {
    if path.is_null() {
        return ptr::null_mut();
    }

    let path_str = match { cstr_to_rust(path) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match DB::open(&path_str) {
        Ok(db) => { box_into_raw_new(db) as *mut c_void },
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_exec(db: *mut c_void, sql: *const c_char) -> bool {
    if db.is_null() || sql.is_null() {
        return false;
    }

    let db = unsafe { &*(db as *mut DB) };
    let sql_str = match { cstr_to_rust(sql) } {
        Some(s) => s,
        None => return false,
    };

    db.exec(&sql_str).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_query(db: *mut c_void, sql: *const c_char) -> *mut c_void {
    if db.is_null() || sql.is_null() {
        return ptr::null_mut();
    }

    let db = unsafe { &mut *(db as *mut DB) };
    let sql_str = match { cstr_to_rust(sql) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match db.query(&sql_str) {
        Ok(result) => { box_into_raw_new(result) as *mut c_void },
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_next_row(result: *mut c_void) -> bool {
    if result.is_null() {
        return false;
    }

    let result = unsafe { &mut *(result as *mut QueryResult) };
    result.next_row().is_some()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_string(result: *mut c_void, column: *const c_char) -> *mut c_char {
    if result.is_null() || column.is_null() {
        return ptr::null_mut();
    }

    let result = unsafe { &mut *(result as *mut QueryResult) };
    let column_str = match { cstr_to_rust(column) } {
        Some(s) => s,
        None => return ptr::null_mut(),
    };

    match result.next_row().and_then(|row| row.get_string(&column_str)) {
        Some(s) => { rust_to_cstr(s) },
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_i64(result: *mut c_void, column: *const c_char) -> i64 {
    if result.is_null() || column.is_null() {
        return 0;
    }

    let result = unsafe { &mut *(result as *mut QueryResult) };
    let column_str = match { cstr_to_rust(column) } {
        Some(s) => s,
        None => return 0,
    };

    result.next_row()
        .and_then(|row| row.get_i64(&column_str))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_get_f64(result: *mut c_void, column: *const c_char) -> f64 {
    if result.is_null() || column.is_null() {
        return 0.0;
    }

    let result = unsafe { &mut *(result as *mut QueryResult) };
    let column_str = match { cstr_to_rust(column) } {
        Some(s) => s,
        None => return 0.0,
    };

    result.next_row()
        .and_then(|row| row.get_f64(&column_str))
        .unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_string(s: *mut c_char) {
    ngenrs_free_ptr(s)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_database(db: *mut c_void) {
    if !db.is_null() {
        unsafe { let _ = Box::from_raw(db as *mut DB); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_db_free_result(result: *mut c_void) {
    if !result.is_null() {
        unsafe { let _ = Box::from_raw(result as *mut QueryResult); }
    }
}