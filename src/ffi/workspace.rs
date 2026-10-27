use crate::ffi::crud::{self, Resource};
use crate::utils::cjson::{handle, parse_input, parse_optional};
use crate::utils::db;
use serde::Deserialize;
use serde_json::{json, Value};
use std::ffi::{c_char, c_int};
use std::path::MAIN_SEPARATOR;

static PROJECT: Resource = Resource {
    db: db::PROJECT_DB,
    table: "projects",
    fields: &[
        "organ_uid",
        "create_by",
        "pro_tpl_uid",
        "cover",
        "name",
        "description",
        "sort",
        "schedule",
        "is_private",
        "is_archived",
        "archive_time",
        "open_begin_time",
        "open_task_private",
        "begin_time",
        "end_time",
        "recycle_time",
        "is_recycled",
        "auto_update_schedule",
    ],
    required: &["name"],
    soft_delete: true,
    updated_at: true,
    search: &["name", "description"],
    default_order: "id DESC",
};

static FEATURE: Resource = Resource {
    db: db::PROJECT_DB,
    table: "project_features",
    fields: &["project_uid", "name", "description"],
    required: &["project_uid"],
    soft_delete: false,
    updated_at: true,
    search: &["name", "description"],
    default_order: "id DESC",
};

static VERSION: Resource = Resource {
    db: db::PROJECT_DB,
    table: "project_versions",
    fields: &[
        "project_uid",
        "name",
        "description",
        "start_time",
        "publish_time",
        "plan_publish_time",
        "schedule",
        "status",
        "features_uid",
    ],
    required: &["project_uid"],
    soft_delete: false,
    updated_at: true,
    search: &["name", "description"],
    default_order: "id DESC",
};

static VERSION_LOG: Resource = Resource {
    db: db::PROJECT_DB,
    table: "project_version_logs",
    fields: &[
        "staff_uid",
        "content",
        "remark",
        "log_type",
        "task_uid",
        "project_uid",
        "features_uid",
    ],
    required: &["project_uid"],
    soft_delete: false,
    updated_at: false,
    search: &["content", "remark"],
    default_order: "id DESC",
};

static LOG: Resource = Resource {
    db: db::PROJECT_DB,
    table: "project_logs",
    fields: &[
        "project_uid",
        "task_uid",
        "staff_uid",
        "content",
        "remark",
        "log_type",
        "action_type",
        "to_staff_uid",
        "is_comment",
    ],
    required: &["project_uid"],
    soft_delete: false,
    updated_at: false,
    search: &["content", "remark"],
    default_order: "id DESC",
};

static DESIGN_FILE: Resource = Resource {
    db: db::DESIGN_DB,
    table: "design_files",
    fields: &[
        "project_uid",
        "organ_uid",
        "flow_uid",
        "type_uid",
        "name",
        "description",
        "create_by",
        "cover",
    ],
    required: &["project_uid", "name"],
    soft_delete: false,
    updated_at: true,
    search: &["name", "description"],
    default_order: "id DESC",
};

static DESIGN_TYPE: Resource = Resource {
    db: db::DESIGN_DB,
    table: "design_types",
    fields: &["type_name", "icon", "extra"],
    required: &["type_name"],
    soft_delete: false,
    updated_at: true,
    search: &["type_name"],
    default_order: "id DESC",
};

static DESIGN_VERSION: Resource = Resource {
    db: db::DESIGN_DB,
    table: "design_versions",
    fields: &[
        "cover",
        "design_uid",
        "child_uid",
        "soft_ver",
        "name",
        "logs",
        "fuid",
        "create_by",
    ],
    required: &["design_uid"],
    soft_delete: false,
    updated_at: true,
    search: &["name", "logs"],
    default_order: "id DESC",
};

macro_rules! crud_api {
    ($list:ident, $get:ident, $create:ident, $update:ident, $delete:ident, $res:expr) => {
        #[no_mangle]
        pub extern "C" fn $list(json_in: *const c_char, json_out: *mut c_char) -> c_int {
            handle(json_in, json_out, || {
                let req: Value = parse_input(json_in)?;
                crud::list(&$res, &req)
            })
        }

        #[no_mangle]
        pub extern "C" fn $get(json_in: *const c_char, json_out: *mut c_char) -> c_int {
            handle(json_in, json_out, || {
                let req: Value = parse_input(json_in)?;
                crud::get(&$res, &req)
            })
        }

        #[no_mangle]
        pub extern "C" fn $create(json_in: *const c_char, json_out: *mut c_char) -> c_int {
            handle(json_in, json_out, || {
                let req: Value = parse_input(json_in)?;
                crud::create(&$res, &req)
            })
        }

        #[no_mangle]
        pub extern "C" fn $update(json_in: *const c_char, json_out: *mut c_char) -> c_int {
            handle(json_in, json_out, || {
                let req: Value = parse_input(json_in)?;
                crud::update(&$res, &req)
            })
        }

        #[no_mangle]
        pub extern "C" fn $delete(json_in: *const c_char, json_out: *mut c_char) -> c_int {
            handle(json_in, json_out, || {
                let req: Value = parse_input(json_in)?;
                crud::delete(&$res, &req)
            })
        }
    };
}

crud_api!(
    post_project_list,
    post_project_get,
    post_project_create,
    post_project_update,
    post_project_delete,
    PROJECT
);
crud_api!(
    post_feature_list,
    post_feature_get,
    post_feature_create,
    post_feature_update,
    post_feature_delete,
    FEATURE
);
crud_api!(
    post_version_list,
    post_version_get,
    post_version_create,
    post_version_update,
    post_version_delete,
    VERSION
);
crud_api!(
    post_version_log_list,
    post_version_log_get,
    post_version_log_create,
    post_version_log_update,
    post_version_log_delete,
    VERSION_LOG
);
crud_api!(
    post_log_list,
    post_log_get,
    post_log_create,
    post_log_update,
    post_log_delete,
    LOG
);
crud_api!(
    post_design_file_list,
    post_design_file_get,
    post_design_file_create,
    post_design_file_update,
    post_design_file_delete,
    DESIGN_FILE
);
crud_api!(
    post_design_type_list,
    post_design_type_get,
    post_design_type_create,
    post_design_type_update,
    post_design_type_delete,
    DESIGN_TYPE
);
crud_api!(
    post_design_version_list,
    post_design_version_get,
    post_design_version_create,
    post_design_version_update,
    post_design_version_delete,
    DESIGN_VERSION
);

// ---------------------------------------------------------------------------
// setup
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct SetupReq {}

#[no_mangle]
pub extern "C" fn post_setup(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    handle(json_in, json_out, || {
        let _: SetupReq = parse_optional(json_in)?;

        db::create_table(db::PROJECT_DB, "projects", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, organ_uid TEXT, create_by TEXT, pro_tpl_uid TEXT, cover TEXT, name TEXT, description TEXT, sort INTEGER, schedule TEXT, is_private INTEGER DEFAULT 0, is_archived INTEGER DEFAULT 0, archive_time TEXT, open_begin_time TEXT, open_task_private INTEGER DEFAULT 0, begin_time TEXT, end_time TEXT, recycle_time TEXT, is_recycled INTEGER DEFAULT 0, auto_update_schedule INTEGER DEFAULT 0, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime')), deleted_at TEXT")?;
        db::create_table(db::PROJECT_DB, "project_features", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, project_uid TEXT, name TEXT, description TEXT, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime'))")?;
        db::create_table(db::PROJECT_DB, "project_versions", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, project_uid TEXT, name TEXT, description TEXT, start_time TEXT, publish_time TEXT, plan_publish_time TEXT, schedule INTEGER, status INTEGER, features_uid TEXT, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime'))")?;
        db::create_table(db::PROJECT_DB, "project_version_logs", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, staff_uid TEXT, content TEXT, remark TEXT, log_type TEXT, task_uid TEXT, project_uid TEXT, features_uid TEXT, created_at TEXT DEFAULT (datetime('now','localtime'))")?;
        db::create_table(db::PROJECT_DB, "project_logs", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, project_uid TEXT, task_uid TEXT, staff_uid TEXT, content TEXT, remark TEXT, log_type TEXT, action_type TEXT, to_staff_uid TEXT, is_comment INTEGER DEFAULT 0, created_at TEXT DEFAULT (datetime('now','localtime'))")?;

        db::create_table(db::DESIGN_DB, "design_files", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, project_uid TEXT, organ_uid TEXT, flow_uid TEXT, type_uid TEXT, name TEXT, description TEXT, create_by TEXT, cover TEXT, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime'))")?;
        db::create_table(db::DESIGN_DB, "design_types", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, type_name TEXT, icon TEXT, extra TEXT, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime'))")?;
        db::create_table(db::DESIGN_DB, "design_versions", "id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE, cover TEXT, design_uid TEXT, child_uid TEXT, soft_ver TEXT, name TEXT, logs TEXT, fuid TEXT, create_by TEXT, created_at TEXT DEFAULT (datetime('now','localtime')), updated_at TEXT DEFAULT (datetime('now','localtime'))")?;

        let project_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_projects_organ_uid ON projects(organ_uid)",
            "CREATE INDEX IF NOT EXISTS idx_projects_create_by ON projects(create_by)",
            "CREATE INDEX IF NOT EXISTS idx_projects_pro_tpl_uid ON projects(pro_tpl_uid)",
            "CREATE INDEX IF NOT EXISTS idx_projects_deleted_at ON projects(deleted_at)",
            "CREATE INDEX IF NOT EXISTS idx_project_features_project_uid ON project_features(project_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_versions_project_uid ON project_versions(project_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_versions_features_uid ON project_versions(features_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_version_logs_staff_uid ON project_version_logs(staff_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_version_logs_task_uid ON project_version_logs(task_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_version_logs_project_uid ON project_version_logs(project_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_version_logs_features_uid ON project_version_logs(features_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_logs_project_uid ON project_logs(project_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_logs_task_uid ON project_logs(task_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_logs_staff_uid ON project_logs(staff_uid)",
            "CREATE INDEX IF NOT EXISTS idx_project_logs_to_staff_uid ON project_logs(to_staff_uid)",
        ];
        for sql in project_indexes {
            db::update(db::PROJECT_DB, sql)?;
        }

        let design_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_design_files_project_uid ON design_files(project_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_files_organ_uid ON design_files(organ_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_files_flow_uid ON design_files(flow_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_files_type_uid ON design_files(type_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_files_create_by ON design_files(create_by)",
            "CREATE INDEX IF NOT EXISTS idx_design_versions_design_uid ON design_versions(design_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_versions_child_uid ON design_versions(child_uid)",
            "CREATE INDEX IF NOT EXISTS idx_design_versions_fuid ON design_versions(fuid)",
            "CREATE INDEX IF NOT EXISTS idx_design_versions_create_by ON design_versions(create_by)",
        ];
        for sql in design_indexes {
            db::update(db::DESIGN_DB, sql)?;
        }

        Ok(json!({
            "tables": 8,
            "indexes": project_indexes.len() + design_indexes.len(),
        }))
    })
}

// ---------------------------------------------------------------------------
// project archive / restore
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn post_project_archive(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    handle(json_in, json_out, || {
        let req: Value = parse_input(json_in)?;
        let where_sql = crud::locate(&req)?;
        let archived = req.get("archived").and_then(Value::as_bool).unwrap_or(true);
        let sql = if archived {
            format!(
                "UPDATE projects SET is_archived = 1, archive_time = {}, updated_at = {} WHERE {}",
                db::now_sql(),
                db::now_sql(),
                where_sql
            )
        } else {
            format!(
                "UPDATE projects SET is_archived = 0, archive_time = NULL, updated_at = {} WHERE {}",
                db::now_sql(),
                where_sql
            )
        };
        db::update(db::PROJECT_DB, &sql)?;
        crud::get(&PROJECT, &req)
    })
}

#[no_mangle]
pub extern "C" fn post_project_restore(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    handle(json_in, json_out, || {
        let req: Value = parse_input(json_in)?;
        let where_sql = crud::locate(&req)?;
        db::update(
            db::PROJECT_DB,
            &format!(
                "UPDATE projects SET deleted_at = NULL, recycle_time = NULL, is_recycled = 0, updated_at = {} WHERE {}",
                db::now_sql(),
                where_sql
            ),
        )?;
        let mut req = req;
        if let Some(obj) = req.as_object_mut() {
            obj.insert("include_deleted".to_string(), Value::Bool(true));
        }
        crud::get(&PROJECT, &req)
    })
}

// ---------------------------------------------------------------------------
// save_design
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct SaveDesignReq {
    project_uid: Option<String>,
    file_name: Option<String>,
    puid: Option<String>,
    uuid: Option<String>,
    ver: Option<String>,
}

fn data_path(value: &Value) -> String {
    value
        .get("data")
        .and_then(|d| d.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn file_path_of(value: &Value) -> String {
    let data = value.get("data").cloned().unwrap_or(Value::Null);
    data.get("file_path")
        .or_else(|| data.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn join(dir: &str, name: &str) -> String {
    format!("{}{}{}", dir, MAIN_SEPARATOR, name)
}

fn parse_name(path: &str) -> (String, String) {
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let base = file.split('.').next().unwrap_or(file);
    let mut parts = base.splitn(2, '_');
    let puid = parts.next().unwrap_or("").to_string();
    let uuid = parts.next().unwrap_or("").to_string();
    (puid, uuid)
}

#[no_mangle]
pub extern "C" fn post_save_design(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    handle(json_in, json_out, || {
        let req: SaveDesignReq = parse_input(json_in)?;
        let project_uid = req
            .project_uid
            .filter(|s| !s.is_empty())
            .ok_or_else(|| (400103, "project_uid is required".to_string()))?;
        let ver = req.ver.unwrap_or_else(|| "26".to_string());

        let run_dir = data_path(&db::sysinfo(b"post_cur_dir", json!({}))
            .map_err(|(c, m)| (500200, format!("{} ({})", m, c)))?);
        if run_dir.is_empty() {
            return Err((500200, "current dir is empty".to_string()));
        }

        let workspace_dir = data_path(
            &db::sysinfo(b"post_create_dir", json!({ "dir_name": "workspace" }))
                .map_err(|(c, m)| (500200, format!("{} ({})", m, c)))?,
        );
        let versions_dir = data_path(
            &db::sysinfo(
                b"post_create_dir",
                json!({ "dir_name": "workspace\\versions" }),
            )
            .map_err(|(c, m)| (500200, format!("{} ({})", m, c)))?,
        );
        if workspace_dir.is_empty() || versions_dir.is_empty() {
            return Err((500200, "workspace dir is empty".to_string()));
        }

        let mut puid = req.puid.unwrap_or_default();
        let mut uuid = req.uuid.unwrap_or_default();

        if puid.is_empty() || uuid.is_empty() {
            if let Ok(value) = db::cdrimex(b"post_current_file_path", json!({ "ver": ver })) {
                let path = file_path_of(&value);
                let (p, u) = parse_name(&path);
                if !p.is_empty() && !u.is_empty() {
                    puid = p;
                    uuid = u;
                }
            }
        }
        if puid.is_empty() {
            puid = project_uid.clone();
        }
        if uuid.is_empty() {
            uuid = uuid::Uuid::new_v4().to_string();
        }

        let file_name = req
            .file_name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{}_{}.cdr", puid, uuid));
        let main_path = join(&workspace_dir, &file_name);

        db::cdrimex(
            b"post_export_file",
            json!({ "file_src": main_path, "ver": ver }),
        )
        .map_err(|(c, m)| (500300, format!("export file failed: {} ({})", m, c)))?;

        let cover = format!(
            "{}.png",
            uuid::Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .chars()
                .take(16)
                .collect::<String>()
        );
        let cover_path = join(&join(&join(&run_dir, "web"), "assets"), &cover);
        db::cdrimex(
            b"post_export_cover",
            json!({ "cover_src": cover_path, "ver": ver }),
        )
        .map_err(|(c, m)| (500300, format!("export cover failed: {} ({})", m, c)))?;

        let existing = db::query(
            db::DESIGN_DB,
            &format!(
                "SELECT * FROM design_files WHERE uuid = {} LIMIT 1",
                db::esc(&uuid)
            ),
        )?;
        if let Some(row) = existing.first() {
            let id = row.get("id").and_then(Value::as_i64).unwrap_or(0);
            if id > 0 {
                db::update(
                    db::DESIGN_DB,
                    &format!(
                        "UPDATE design_files SET cover = {}, updated_at = {} WHERE id = {}",
                        db::esc(&cover),
                        db::now_sql(),
                        id
                    ),
                )?;
            }
        } else {
            db::insert(
                db::DESIGN_DB,
                &format!(
                    "INSERT INTO design_files (uuid, project_uid, name, cover, created_at, updated_at) VALUES ({}, {}, {}, {}, {}, {})",
                    db::esc(&uuid),
                    db::esc(&project_uid),
                    db::esc(&file_name),
                    db::esc(&cover),
                    db::now_sql(),
                    db::now_sql(),
                ),
            )?;
        }

        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let version_name = format!("{}_{}.cdr", uuid, stamp);
        let version_path = join(&versions_dir, &version_name);
        db::cdrimex(
            b"post_export_file",
            json!({ "file_src": version_path, "ver": ver }),
        )
        .map_err(|(c, m)| (500300, format!("export version failed: {} ({})", m, c)))?;

        // 通过 ostfil 归档版本文件，得到存储 uuid
        let stored = db::ostfil(b"post_storage", json!({ "file_path": version_path }))
            .map_err(|(c, m)| (500400, format!("ostfil storage failed: {} ({})", m, c)))?;
        let storage_uuid = stored
            .get("data")
            .and_then(|d| d.get("uuid"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if storage_uuid.is_empty() {
            return Err((500400, "ostfil returned empty uuid".to_string()));
        }

        let version_uuid = uuid::Uuid::new_v4().to_string();
        db::insert(
            db::DESIGN_DB,
            &format!(
                "INSERT INTO design_versions (uuid, cover, design_uid, child_uid, soft_ver, name, logs, fuid, create_by, created_at, updated_at) \
                 VALUES ({}, {}, {}, '', {}, {}, '自动保存', {}, '', {}, {})",
                db::esc(&version_uuid),
                db::esc(&cover),
                db::esc(&uuid),
                db::esc(&ver),
                db::esc(&format!("v{}", stamp)),
                db::esc(&storage_uuid),
                db::now_sql(),
                db::now_sql(),
            ),
        )
        .map_err(|(c, m)| (500400, format!("insert design version failed: {} ({})", m, c)))?;

        Ok(json!({
            "uuid": uuid,
            "puid": puid,
            "file_name": file_name,
            "version_path": version_path,
            "version_uuid": version_uuid,
            "storage_uuid": storage_uuid,
            "cover": cover,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name_splits_puid_and_uuid() {
        let (p, u) = parse_name("D:\\a\\p_123.cdr");
        assert_eq!(p, "p");
        assert_eq!(u, "123");
    }

    #[test]
    fn parse_name_without_separator() {
        let (p, u) = parse_name("D:\\a\\plain.cdr");
        assert_eq!(p, "plain");
        assert_eq!(u, "");
    }

    #[test]
    fn join_uses_separator() {
        assert!(join("a", "b").contains(MAIN_SEPARATOR));
    }
}
