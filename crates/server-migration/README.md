# 迁移项目使用指南

## 新建迁移

```bash
sea-orm-cli migrate generate -d crates/server-migration/ NAME_OF_MIGRATION
```

## 根据数据库表生成`entity`文件

```bash
sea-orm-cli generate entity -o crates/server-entity/src/ -l
```

### 其他参数参考

- `--with-serde both` 为`Entity`生成`Serialize / Deserialize`
- `--serde-skip-deserializing-primary-key` 为`Entity`生成`Deserialize`时为主键生成`#[serde(skip_deserializing)]`