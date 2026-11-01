# workspace_lib MCP 服务

> 状态：接口文档（对应 `workspace_lib` 1.0.1 的 FFI）
> 更新日期：2026-10-28
> 打包产物：`target/release/workspace_lib.dll`
> 底层接口：`post_setup`、8 组 CRUD、`post_project_archive` / `post_project_restore` / `post_save_design`

本文把 `workspace_lib` 的 44 个 FFI 接口整理为 MCP 工具：工具名、输入结构、
错误语义与实现注意事项。MCP server 由宿主实现，底层通过 FFI 调用
`workspace_lib.dll`，也可以直接依赖本 crate 的 `rlib`。

`workspace_lib` 封装项目 / 工作区业务数据、设计文件与设计版本，以及保存编排；
页面不直连 `database.dll`、`sysinfo.dll` 或 `libostfil.dll`。

---

## 1. 工具映射规则

| 项 | 规则 |
| --- | --- |
| 工具名 | `workspace_` + FFI-Symbol（如 `workspace_project_list`） |
| 底层符号 | `post_` + FFI-Symbol（如 `post_project_list`） |
| HTTP（可选） | `POST /workspace_lib` + `FFI-Symbol: project_list` |
| 输出外壳 | `{ "code": 200, "data": {}, "msg": "success" }` |
| 只读工具 | `*_list` / `*_get` |
| 写工具 | `*_create` / `*_update` / `*_delete` / `setup` / `*_archive` / `*_restore` / `save_design` |

MCP 侧建议：

- 工具结果同时返回 `content`（摘要）与 `structuredContent`（原始 `{code,data,msg}`）；
- `code=200` 时 `isError=false`；其余 `isError=true`，文本使用 `msg`；
- FFI 返回 `-1` / `-2` 视为 server 内部错误。

---

## 2. 调用约定

### 2.1 FFI

```c
int post_project_list(const char *json_in, char *json_out);
```

| 项 | 约定 |
| --- | --- |
| `json_in` | NUL 结尾 UTF-8 JSON；`setup` 可为空 / `null` |
| `json_out` | 至少 65536 字节（含结尾 NUL） |
| 返回值 | `0` 已写入；`-1` 输出指针为空；`-2` 输出超过 64 KiB |
| 字段命名 | `snake_case` |
| 时间 | 本地时间字符串 |
| 输出缓冲 | 64 KiB |

### 2.2 错误码

| code | 含义 |
| --- | --- |
| `200` | 成功 |
| `400100` | `json_in` 为空 |
| `400101` | 入参不是合法 UTF-8 |
| `400102` | JSON 解析失败 / 请求不是对象 |
| `400103` | 必填字段为空 / 没有可更新字段 |
| `400104` | 非法参数（未知字段、非法过滤字段、缺 `id`/`uuid`、非法 `order`） |
| `404100` | 资源不存在（项目 / 特性 / 版本 / 设计文件等统一码） |
| `409100` | 唯一键冲突（`uuid`） |
| `500100` | 打开数据库失败 |
| `500101` | SQL 执行失败 |
| `500102` | 输出超过 64 KiB |
| `500200` | 当前目录 / 工作目录 / 版本目录准备失败 |
| `500300` | 设计文件 / 封面导出失败 |
| `500400` | `ostfil` 归档失败或记录写入失败 |

---

## 3. 资源与字段

8 个资源共用一套 CRUD 语义：

| 资源 | 表 | 字段 | 必填 | 软删除 | 搜索字段 |
| --- | --- | --- | --- | --- | --- |
| `project` | `projects` | `organ_uid`、`create_by`、`pro_tpl_uid`、`cover`、`name`、`description`、`sort`、`schedule`、`is_private`、`is_archived`、`archive_time`、`open_begin_time`、`open_task_private`、`begin_time`、`end_time`、`recycle_time`、`is_recycled`、`auto_update_schedule` | `name` | 是 | `name`、`description` |
| `feature` | `project_features` | `project_uid`、`name`、`description` | `project_uid` | 否 | `name`、`description` |
| `version` | `project_versions` | `project_uid`、`name`、`description`、`start_time`、`publish_time`、`plan_publish_time`、`schedule`、`status`、`features_uid` | `project_uid` | 否 | `name`、`description` |
| `version_log` | `project_version_logs` | `staff_uid`、`content`、`remark`、`log_type`、`task_uid`、`project_uid`、`features_uid` | `project_uid` | 否 | `content`、`remark` |
| `log` | `project_logs` | `project_uid`、`task_uid`、`staff_uid`、`content`、`remark`、`log_type`、`action_type`、`to_staff_uid`、`is_comment` | `project_uid` | 否 | `content`、`remark` |
| `design_file` | `design_files` | `project_uid`、`organ_uid`、`flow_uid`、`type_uid`、`name`、`description`、`create_by`、`cover` | `project_uid`、`name` | 否 | `name`、`description` |
| `design_type` | `design_types` | `type_name`、`icon`、`extra` | `type_name` | 否 | `type_name` |
| `design_version` | `design_versions` | `cover`、`design_uid`、`child_uid`、`soft_ver`、`name`、`logs`、`fuid`、`create_by` | `design_uid` | 否 | `name`、`logs` |

每个资源都有：

```text
workspace_<resource>_list
workspace_<resource>_get
workspace_<resource>_create
workspace_<resource>_update
workspace_<resource>_delete
```

---

## 4. 通用输入结构

### 4.1 `list`

```json
{
  "page": 1,
  "page_size": 20,
  "include_deleted": false,
  "order": "id DESC",
  "filter": {
    "project_uid": "p-1",
    "name_like": "海报",
    "keyword": "海报"
  }
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `page` | int | 默认 1 |
| `page_size` | int | 默认 20，上限 200 |
| `limit` | int | 替代分页，范围 1–1000；设置后 offset=0 |
| `include_deleted` | bool | 仅对软删除资源有效 |
| `order` | string | 逗号分隔；字段 + 可选 `asc` / `desc`；非法值回退默认 `id DESC` |
| `filter` | object | 等值过滤；字段必须在资源字段内 |
| `filter.keyword` | string | 在资源 `search` 字段内做 `LIKE` |
| `filter.<field>_like` | string | 对指定字段做 `LIKE` |

响应：

```json
{ "list": [], "page": 1, "page_size": 20, "total": 0 }
```

### 4.2 `get`

```json
{ "id": 12, "include_deleted": false }
```

或 `{ "uuid": "..." }`；至少一个。响应为单条记录。

### 4.3 `create`

```json
{
  "uuid": "可选",
  "name": "必填字段示例",
  "description": "..."
}
```

- 必填字段见资源表；
- 可选 `uuid`，缺省自动生成；
- 字段必须在资源字段内，未知字段返回 `400104`；
- 响应：

```json
{ "id": 1, "uuid": "...", "record": {} }
```

### 4.4 `update`

```json
{ "id": 12, "name": "新名称", "description": "..." }
```

- 定位 `id` 或 `uuid`；
- 只接受资源字段，未知字段 `400104`；
- 至少一个字段，否则 `400103`；
- 响应为更新后的记录。

### 4.5 `delete`

```json
{ "id": 12, "hard": false }
```

- 软删除资源默认写 `deleted_at`；
- `hard=true` 物理删除；
- 响应 `{ "deleted": true, "hard": false }`。

---

## 5. 工具清单（44）

### 5.1 初始化

| 工具 | FFI-Symbol | 说明 | 输入 |
| --- | --- | --- | --- |
| `workspace_setup` | `setup` | 建 8 张表与索引（幂等） | 空 / `null` |

### 5.2 项目

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_project_list` | `project_list` | 项目分页列表 |
| `workspace_project_get` | `project_get` | 项目详情 |
| `workspace_project_create` | `project_create` | 新建项目（`name` 必填） |
| `workspace_project_update` | `project_update` | 更新项目 |
| `workspace_project_delete` | `project_delete` | 删除项目（软删除） |
| `workspace_project_archive` | `project_archive` | 归档 / 取消归档（`archived`，默认 `true`） |
| `workspace_project_restore` | `project_restore` | 从回收站恢复 |

### 5.3 项目特性

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_feature_list` | `feature_list` | 特性列表 |
| `workspace_feature_get` | `feature_get` | 特性详情 |
| `workspace_feature_create` | `feature_create` | 新建特性（`project_uid` 必填） |
| `workspace_feature_update` | `feature_update` | 更新特性 |
| `workspace_feature_delete` | `feature_delete` | 删除特性 |

### 5.4 项目版本

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_version_list` | `version_list` | 版本列表 |
| `workspace_version_get` | `version_get` | 版本详情 |
| `workspace_version_create` | `version_create` | 新建版本（`project_uid` 必填） |
| `workspace_version_update` | `version_update` | 更新版本 |
| `workspace_version_delete` | `version_delete` | 删除版本 |

### 5.5 版本日志 / 项目日志

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_version_log_list` | `version_log_list` | 版本日志列表 |
| `workspace_version_log_get` | `version_log_get` | 版本日志详情 |
| `workspace_version_log_create` | `version_log_create` | 新建版本日志 |
| `workspace_version_log_update` | `version_log_update` | 更新版本日志 |
| `workspace_version_log_delete` | `version_log_delete` | 删除版本日志 |
| `workspace_log_list` | `log_list` | 项目动态列表 |
| `workspace_log_get` | `log_get` | 项目动态详情 |
| `workspace_log_create` | `log_create` | 新建动态 / 评论 |
| `workspace_log_update` | `log_update` | 更新动态 |
| `workspace_log_delete` | `log_delete` | 删除动态 |

### 5.6 设计文件

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_design_file_list` | `design_file_list` | 设计文件列表 |
| `workspace_design_file_get` | `design_file_get` | 设计文件详情 |
| `workspace_design_file_create` | `design_file_create` | 新建设计文件（`project_uid`、`name` 必填） |
| `workspace_design_file_update` | `design_file_update` | 更新设计文件 |
| `workspace_design_file_delete` | `design_file_delete` | 删除设计文件 |

### 5.7 设计类型 / 设计版本

| 工具 | FFI-Symbol | 说明 |
| --- | --- | --- |
| `workspace_design_type_list` | `design_type_list` | 设计类型列表 |
| `workspace_design_type_get` | `design_type_get` | 设计类型详情 |
| `workspace_design_type_create` | `design_type_create` | 新建类型（`type_name` 必填） |
| `workspace_design_type_update` | `design_type_update` | 更新类型 |
| `workspace_design_type_delete` | `design_type_delete` | 删除类型 |
| `workspace_design_version_list` | `design_version_list` | 设计版本列表 |
| `workspace_design_version_get` | `design_version_get` | 设计版本详情 |
| `workspace_design_version_create` | `design_version_create` | 新建版本（`design_uid` 必填） |
| `workspace_design_version_update` | `design_version_update` | 更新版本 |
| `workspace_design_version_delete` | `design_version_delete` | 删除版本 |

### 5.8 保存编排

| 工具 | FFI-Symbol | 说明 | 输入 |
| --- | --- | --- | --- |
| `workspace_save_design` | `save_design` | 保存当前设计到工作区 | `{project_uid, file_name?, puid?, uuid?, ver?}` |

> `save_design` 仍调用已移除的 `cdrimex` 导出 / 封面接口，当前不可用；
> 目录准备、写库与 `ostfil` 归档逻辑保留。以源码为准。

---

## 6. MCP 工具 Schema 示例

### 6.1 列表工具

```json
{
  "name": "workspace_project_list",
  "description": "分页查询项目，支持关键词、归档 / 回收站过滤。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "page": { "type": "integer", "minimum": 1, "default": 1 },
      "page_size": { "type": "integer", "minimum": 1, "maximum": 200, "default": 20 },
      "limit": { "type": "integer", "minimum": 1, "maximum": 1000 },
      "include_deleted": { "type": "boolean", "default": false },
      "order": { "type": "string", "default": "id DESC" },
      "filter": {
        "type": "object",
        "properties": {
          "keyword": { "type": "string" },
          "name": { "type": "string" },
          "name_like": { "type": "string" },
          "is_archived": { "type": "integer" },
          "is_recycled": { "type": "integer" }
        },
        "additionalProperties": true
      }
    },
    "additionalProperties": false
  }
}
```

### 6.2 创建工具

```json
{
  "name": "workspace_design_file_create",
  "description": "新建设计文件；project_uid 与 name 必填。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "uuid": { "type": "string" },
      "project_uid": { "type": "string", "minLength": 1 },
      "organ_uid": { "type": "string" },
      "flow_uid": { "type": "string" },
      "type_uid": { "type": "string" },
      "name": { "type": "string", "minLength": 1 },
      "description": { "type": "string" },
      "create_by": { "type": "string" },
      "cover": { "type": "string" }
    },
    "required": ["project_uid", "name"],
    "additionalProperties": false
  }
}
```

### 6.3 删除工具

```json
{
  "name": "workspace_project_delete",
  "description": "删除项目；默认软删除进回收站，hard=true 时物理删除。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "integer" },
      "uuid": { "type": "string" },
      "hard": { "type": "boolean", "default": false }
    },
    "anyOf": [
      { "required": ["id"] },
      { "required": ["uuid"] }
    ],
    "additionalProperties": false
  }
}
```

### 6.4 归档 / 恢复

```json
{
  "name": "workspace_project_archive",
  "description": "归档或取消归档项目。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "integer" },
      "uuid": { "type": "string" },
      "archived": { "type": "boolean", "default": true }
    },
    "anyOf": [
      { "required": ["id"] },
      { "required": ["uuid"] }
    ],
    "additionalProperties": false
  }
}
```

---

## 7. 输出

| 工具类型 | `data` |
| --- | --- |
| `setup` | `{ "tables": 8, "indexes": 24 }` |
| `*_list` | `{ "list": [], "page": 1, "page_size": 20, "total": 0 }` |
| `*_get` / `*_update` / `*_archive` / `*_restore` | 单条记录 |
| `*_create` | `{ "id": 1, "uuid": "...", "record": {} }` |
| `*_delete` | `{ "deleted": true, "hard": false }` |
| `save_design` | `{uuid, puid, file_name, version_path, version_uuid, storage_uuid, cover}` |

---

## 8. 实现指南

### 8.1 FFI 调用

```c
int post_project_list(const char *json_in, char *json_out);
```

Rust（`libloading`）示例：

```rust
let lib = unsafe { Library::new("workspace_lib.dll")? };
let f: Symbol<unsafe extern "C" fn(*const c_char, *mut c_char) -> i32> =
    unsafe { lib.get(b"post_project_list\0")? };
let input = CString::new(r#"{"page":1,"page_size":20}"#)?;
let mut out = vec![0 as c_char; 64 * 1024];
let ret = unsafe { f(input.as_ptr(), out.as_mut_ptr()) };
assert_eq!(ret, 0);
```

### 8.2 缓冲区

- 固定 **64 KiB**（含 NUL）；
- `-2` 时不保证内容完整，应扩容重试或报内部错误；
- `-1` 表示 `json_out` 为空，属于 server 编码错误。

### 8.3 线程与超时

- 所有接口同步执行；`save_design` 会调用 `sysinfo` / `cdrimex` / `ostfil`，可能较慢；
- 建议 MCP server 用 `spawn_blocking` 包住 FFI 调用；
- `save_design` 建议超时 ≥ 60 秒；普通 CRUD 10 秒量级。

### 8.4 安全

- 调用方不接触 SQL；字段、过滤与排序都在库内白名单校验；
- `save_design` 会写入工作区目录与 `libostfil`，属于有副作用的写操作；
- 项目、日志内容可能包含用户数据，输出应遵循宿主脱敏策略。

---

## 9. 资源与提示词（可选）

| URI | 内容 |
| --- | --- |
| `workspace://schema` | 8 张表与字段说明（来自 `ui/workspace/db.md`） |
| `workspace://projects` | 项目列表（`project_list` 的缓存） |

提示词示例：

```json
{
  "name": "create-project",
  "description": "创建一个项目并返回项目 uuid",
  "arguments": [
    { "name": "name", "required": true },
    { "name": "description", "required": false }
  ]
}
```

---

## 10. 版本与兼容

| 项 | 值 |
| --- | --- |
| 包名 / 版本 | `workspace_lib` / 1.0.1 |
| 发布产物 | `workspace_lib.dll` |
| 端点数 | 44 |
| 平台 | Windows（`platform-exports.windows`） |
| 输出缓冲 | 64 KiB |

升级时：

- 新增资源沿用 `list/get/create/update/delete` 命名；
- 新增字段必须加入资源白名单，否则更新 / 过滤会被拒绝；
- 错误码只新增，不复用。
