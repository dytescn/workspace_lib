use libloading::Library;
use serde_json::Value;
use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::OnceLock;

pub const PROJECT_DB: &str = "./project";
pub const DESIGN_DB: &str = "./design";
pub const ROUTER_DB: &str = "./router";

const OUT_BUF_SIZE: usize = 65536;

#[cfg(windows)]
const DATABASE_LIB: &str = "./libary/database.dll";
#[cfg(target_os = "linux")]
const DATABASE_LIB: &str = "./libary/database.so";
#[cfg(target_os = "macos")]
const DATABASE_LIB: &str = "./libary/database.dylib";

#[cfg(windows)]
const SYSINFO_LIB: &str = "./libary/sysinfo.dll";
#[cfg(target_os = "linux")]
const SYSINFO_LIB: &str = "./libary/sysinfo.so";
#[cfg(target_os = "macos")]
const SYSINFO_LIB: &str = "./libary/sysinfo.dylib";

#[cfg(windows)]
const CDRIMEX_LIB: &str = "./libary/cdrimex.dll";
#[cfg(target_os = "linux")]
const CDRIMEX_LIB: &str = "./libary/cdrimex.so";
#[cfg(target_os = "macos")]
const CDRIMEX_LIB: &str = "./libary/cdrimex.dylib";

#[cfg(windows)]
const OSTFIL_LIB: &str = "./libary/libostfil.dll";
#[cfg(target_os = "linux")]
const OSTFIL_LIB: &str = "./libary/libostfil.so";
#[cfg(target_os = "macos")]
const OSTFIL_LIB: &str = "./libary/libostfil.dylib";

type FfiFn = unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int;

static DB_LIB: OnceLock<Library> = OnceLock::new();
static SYSINFO: OnceLock<Library> = OnceLock::new();
static CDRIMEX: OnceLock<Library> = OnceLock::new();
static OSTFIL: OnceLock<Library> = OnceLock::new();

fn load(slot: &'static OnceLock<Library>, path: &str) -> Result<&'static Library, (i32, String)> {
    if let Some(lib) = slot.get() {
        return Ok(lib);
    }
    let lib = unsafe { Library::new(path) }
        .map_err(|e| (500100, format!("load {} failed: {}", path, e)))?;
    let _ = slot.set(lib);
    Ok(slot.get().unwrap())
}

fn call_with(lib: &Library, symbol: &[u8], body: Value) -> Result<Value, (i32, String)> {
    let func: libloading::Symbol<'_, FfiFn> = unsafe {
        lib.get(symbol)
            .map_err(|e| (500101, format!("dlsym failed: {}", e)))?
    };
    let req = CString::new(body.to_string())
        .map_err(|_| (400102, "request json contains \\0".to_string()))?;
    let mut out = vec![0u8; OUT_BUF_SIZE];
    let ret = unsafe { func(req.as_ptr(), out.as_mut_ptr() as *mut c_char) };

    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .map_err(|e| (500101, format!("invalid utf-8 response: {}", e)))?;
    if text.is_empty() {
        return Err((500101, format!("library returned empty (ret={})", ret)));
    }
    let value: Value = serde_json::from_str(text)
        .map_err(|e| (500101, format!("invalid response: {} ({})", e, text)))?;
    let code = value.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 200 && code != 0 {
        let msg = value
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_string();
        if msg.contains("UNIQUE constraint failed") {
            return Err((409100, msg));
        }
        return Err((500101, msg));
    }
    Ok(value)
}

fn db_lib() -> Result<&'static Library, (i32, String)> {
    load(&DB_LIB, DATABASE_LIB)
}

pub fn query(db: &str, sql: &str) -> Result<Vec<Value>, (i32, String)> {
    let value = call_with(
        db_lib()?,
        b"post_db_query",
        serde_json::json!({ "path": db, "sql": sql }),
    )?;
    Ok(value
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub fn query_count(db: &str, sql: &str) -> Result<i64, (i32, String)> {
    Ok(query(db, sql)?
        .first()
        .and_then(|row| row.get("n"))
        .and_then(Value::as_i64)
        .unwrap_or(0))
}

pub fn execute(symbol: &[u8], db: &str, sql: &str) -> Result<(), (i32, String)> {
    call_with(
        db_lib()?,
        symbol,
        serde_json::json!({ "path": db, "sql": sql }),
    )?;
    Ok(())
}

pub fn update(db: &str, sql: &str) -> Result<(), (i32, String)> {
    execute(b"post_db_update", db, sql)
}

pub fn insert(db: &str, sql: &str) -> Result<(), (i32, String)> {
    execute(b"post_db_insert", db, sql)
}

pub fn delete(db: &str, sql: &str) -> Result<(), (i32, String)> {
    execute(b"post_db_delete", db, sql)
}

pub fn create_table(db: &str, table: &str, schema: &str) -> Result<(), (i32, String)> {
    call_with(
        db_lib()?,
        b"post_db_create",
        serde_json::json!({ "path": db, "table": table, "schema": schema }),
    )?;
    Ok(())
}

pub fn sysinfo(symbol: &[u8], body: Value) -> Result<Value, (i32, String)> {
    call_with(load(&SYSINFO, SYSINFO_LIB)?, symbol, body)
}

pub fn cdrimex(symbol: &[u8], body: Value) -> Result<Value, (i32, String)> {
    call_with(load(&CDRIMEX, CDRIMEX_LIB)?, symbol, body)
}

pub fn ostfil(symbol: &[u8], body: Value) -> Result<Value, (i32, String)> {
    call_with(load(&OSTFIL, OSTFIL_LIB)?, symbol, body)
}

pub fn esc(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn now_sql() -> &'static str {
    "datetime('now','localtime')"
}
