use crate::utils::db;
use serde_json::{json, Map, Value};

pub struct Resource {
    pub db: &'static str,
    pub table: &'static str,
    pub fields: &'static [&'static str],
    pub required: &'static [&'static str],
    pub soft_delete: bool,
    pub updated_at: bool,
    pub search: &'static [&'static str],
    pub default_order: &'static str,
}

fn sql_value(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(db::esc(s)),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "1" } else { "0" }.to_string()),
        _ => None,
    }
}

pub fn locate(req: &Value) -> Result<String, (i32, String)> {
    if let Some(id) = req.get("id").and_then(Value::as_i64) {
        if id > 0 {
            return Ok(format!("id = {}", id));
        }
    }
    if let Some(uuid) = req.get("uuid").and_then(Value::as_str) {
        if !uuid.is_empty() {
            return Ok(format!("uuid = {}", db::esc(uuid)));
        }
    }
    Err((400104, "id or uuid is required".to_string()))
}

fn deleted_clause(res: &Resource, include: bool) -> &'static str {
    if !res.soft_delete || include {
        "1 = 1"
    } else {
        "deleted_at IS NULL"
    }
}

fn build_where(res: &Resource, filter: &Value, include_deleted: bool) -> Result<String, (i32, String)> {
    let mut parts = vec![deleted_clause(res, include_deleted).to_string()];

    let Some(obj) = filter.as_object() else {
        return Ok(parts.join(" AND "));
    };

    for (key, value) in obj {
        if key == "keyword" {
            let keyword = value.as_str().unwrap_or("");
            if !keyword.is_empty() {
                let pattern = db::esc(&format!("%{}%", keyword));
                let mut ors = Vec::new();
                for field in res.search {
                    ors.push(format!("{} LIKE {}", field, pattern));
                }
                if !ors.is_empty() {
                    parts.push(format!("({})", ors.join(" OR ")));
                }
            }
            continue;
        }

        if let Some(base) = key.strip_suffix("_like") {
            if !res.fields.contains(&base) {
                return Err((400104, format!("filter field not allowed: {}", base)));
            }
            let text = value.as_str().unwrap_or("");
            parts.push(format!("{} LIKE {}", base, db::esc(&format!("%{}%", text))));
            continue;
        }

        if !res.fields.contains(&key.as_str()) {
            return Err((400104, format!("filter field not allowed: {}", key)));
        }
        if let Some(sql) = sql_value(value) {
            parts.push(format!("{} = {}", key, sql));
        }
    }

    Ok(parts.join(" AND "))
}

fn valid_order(res: &Resource, order: &str) -> bool {
    for part in order.split(',') {
        let mut words = part.trim().split_whitespace();
        let Some(field) = words.next() else {
            return false;
        };
        let allowed = field == "id"
            || field == "created_at"
            || (field == "updated_at" && res.updated_at)
            || res.fields.contains(&field);
        if !allowed {
            return false;
        }
        match words.next() {
            None => {}
            Some(dir) if dir.eq_ignore_ascii_case("asc") || dir.eq_ignore_ascii_case("desc") => {}
            _ => return false,
        }
        if words.next().is_some() {
            return false;
        }
    }
    true
}

pub fn list(res: &Resource, req: &Value) -> Result<Value, (i32, String)> {
    let include_deleted = req
        .get("include_deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let filter = req.get("filter").cloned().unwrap_or(Value::Null);
    let where_sql = build_where(res, &filter, include_deleted)?;

    let limit = req.get("limit").and_then(Value::as_i64);
    let page = req.get("page").and_then(Value::as_i64).unwrap_or(1).max(1);
    let page_size = match limit {
        Some(v) => v.clamp(1, 1000),
        None => req
            .get("page_size")
            .and_then(Value::as_i64)
            .unwrap_or(20)
            .clamp(1, 200),
    };
    let offset = if limit.is_some() { 0 } else { (page - 1) * page_size };

    let order = req
        .get("order")
        .and_then(Value::as_str)
        .filter(|s| valid_order(res, s))
        .unwrap_or(res.default_order);

    let total = db::query_count(
        res.db,
        &format!("SELECT count(*) AS n FROM {} WHERE {}", res.table, where_sql),
    )?;
    let list = db::query(
        res.db,
        &format!(
            "SELECT * FROM {} WHERE {} ORDER BY {} LIMIT {} OFFSET {}",
            res.table, where_sql, order, page_size, offset
        ),
    )?;

    Ok(json!({
        "list": list,
        "page": page,
        "page_size": page_size,
        "total": total,
    }))
}

pub fn get(res: &Resource, req: &Value) -> Result<Value, (i32, String)> {
    let include_deleted = req
        .get("include_deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let where_sql = format!(
        "{} AND {}",
        locate(req)?,
        deleted_clause(res, include_deleted)
    );
    db::query(
        res.db,
        &format!("SELECT * FROM {} WHERE {} LIMIT 1", res.table, where_sql),
    )?
    .into_iter()
    .next()
    .ok_or_else(|| (404100, format!("{} not found", res.table)))
}

pub fn create(res: &Resource, req: &Value) -> Result<Value, (i32, String)> {
    let obj = req
        .as_object()
        .ok_or_else(|| (400102, "request must be an object".to_string()))?;

    for field in res.required {
        let ok = obj
            .get(*field)
            .and_then(Value::as_str)
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !ok {
            return Err((400103, format!("{} is required", field)));
        }
    }

    let uuid = obj
        .get("uuid")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let mut columns = vec!["uuid".to_string()];
    let mut values = vec![db::esc(&uuid)];

    for field in res.fields {
        if let Some(value) = obj.get(*field) {
            if let Some(sql) = sql_value(value) {
                columns.push((*field).to_string());
                values.push(sql);
            }
        }
    }

    db::insert(
        res.db,
        &format!(
            "INSERT INTO {} ({}) VALUES ({})",
            res.table,
            columns.join(", "),
            values.join(", ")
        ),
    )?;

    let row = db::query(
        res.db,
        &format!(
            "SELECT * FROM {} WHERE uuid = {} LIMIT 1",
            res.table,
            db::esc(&uuid)
        ),
    )?
    .into_iter()
    .next()
    .unwrap_or(Value::Null);

    Ok(json!({
        "id": row.get("id").cloned().unwrap_or(Value::Null),
        "uuid": uuid,
        "record": row,
    }))
}

pub fn update(res: &Resource, req: &Value) -> Result<Value, (i32, String)> {
    let obj = req
        .as_object()
        .ok_or_else(|| (400102, "request must be an object".to_string()))?;
    let where_sql = locate(req)?;

    let mut sets = Vec::new();
    for (key, value) in obj {
        if key == "id" || key == "uuid" || key == "include_deleted" {
            continue;
        }
        if !res.fields.contains(&key.as_str()) {
            return Err((400104, format!("field not allowed: {}", key)));
        }
        if let Some(sql) = sql_value(value) {
            sets.push(format!("{} = {}", key, sql));
        }
    }
    if sets.is_empty() {
        return Err((400103, "no fields to update".to_string()));
    }
    if res.updated_at {
        sets.push(format!("updated_at = {}", db::now_sql()));
    }

    db::update(
        res.db,
        &format!(
            "UPDATE {} SET {} WHERE {}",
            res.table,
            sets.join(", "),
            where_sql
        ),
    )?;

    db::query(
        res.db,
        &format!("SELECT * FROM {} WHERE {} LIMIT 1", res.table, where_sql),
    )?
    .into_iter()
    .next()
    .ok_or_else(|| (404100, format!("{} not found", res.table)))
}

pub fn delete(res: &Resource, req: &Value) -> Result<Value, (i32, String)> {
    let hard = req.get("hard").and_then(Value::as_bool).unwrap_or(false);
    let where_sql = locate(req)?;

    if res.soft_delete && !hard {
        let mut sets = vec![format!("deleted_at = {}", db::now_sql())];
        if res.updated_at {
            sets.push(format!("updated_at = {}", db::now_sql()));
        }
        db::update(
            res.db,
            &format!(
                "UPDATE {} SET {} WHERE {}",
                res.table,
                sets.join(", "),
                where_sql
            ),
        )?;
    } else {
        db::delete(
            res.db,
            &format!("DELETE FROM {} WHERE {}", res.table, where_sql),
        )?;
    }

    Ok(json!({ "deleted": true, "hard": hard }))
}

pub fn map_of(req: &Value) -> Option<&Map<String, Value>> {
    req.as_object()
}

#[cfg(test)]
mod tests {
    use super::*;

    static RES: Resource = Resource {
        db: "./project",
        table: "projects",
        fields: &["name", "description", "schedule"],
        required: &["name"],
        soft_delete: true,
        updated_at: true,
        search: &["name", "description"],
        default_order: "id DESC",
    };

    #[test]
    fn where_builds_filters() {
        let filter = json!({ "name": "a", "keyword": "b", "schedule_like": "c" });
        let sql = build_where(&RES, &filter, false).unwrap();
        assert!(sql.contains("deleted_at IS NULL"));
        assert!(sql.contains("name = 'a'"));
        assert!(sql.contains("name LIKE '%b%'"));
        assert!(sql.contains("schedule LIKE '%c%'"));
    }

    #[test]
    fn where_rejects_unknown_field() {
        let filter = json!({ "bad": "x" });
        assert!(matches!(build_where(&RES, &filter, false), Err((400104, _))));
    }

    #[test]
    fn order_allows_fields_and_direction() {
        assert!(valid_order(&RES, "id DESC"));
        assert!(valid_order(&RES, "name ASC, updated_at DESC"));
        assert!(!valid_order(&RES, "name; DROP TABLE"));
        assert!(!valid_order(&RES, "bad DESC"));
    }

    #[test]
    fn locate_requires_id_or_uuid() {
        assert!(locate(&json!({})).is_err());
        assert_eq!(locate(&json!({"id": 2})).unwrap(), "id = 2");
    }
}

#[allow(dead_code)]
fn _unused(map: Option<&Map<String, Value>>) {
    let _ = map;
}
