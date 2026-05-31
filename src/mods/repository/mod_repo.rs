use std::sync::Arc;

use turso::{params, Builder, Connection, Database, Value};

type ModRowValues = (
    String,
    String,
    String,
    Value,
    Value,
    Value,
    Value,
    Value,
    Value,
);

use crate::config::Config;
use crate::error::ModManagerError;
use crate::mods::types::{ModEntry, ModEntryKind, ModStatus, ReconciledMod};

pub trait ModRepository: Send + Sync {
    fn get_mods(&self, game_registry_id: &str) -> Result<Vec<ReconciledMod>, ModManagerError>;
    fn upsert_mod(
        &self,
        game_registry_id: &str,
        mod_entry: &ReconciledMod,
    ) -> Result<(), ModManagerError>;
    fn remove_mod(&self, game_registry_id: &str, name: &str) -> Result<(), ModManagerError>;
    fn set_mods(
        &self,
        game_registry_id: &str,
        mods: &[ReconciledMod],
    ) -> Result<(), ModManagerError>;
}

pub struct TursoModRepository {
    db: Database,
    runtime: tokio::runtime::Runtime,
}

impl TursoModRepository {
    pub fn new(config: &Config) -> Result<Arc<Self>, ModManagerError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;

        let db_path = config.db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;
        }

        log::info!("Opening database at {}", db_path.display());

        let db = runtime.block_on(async {
            Builder::new_local(db_path.to_str().unwrap())
                .build()
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))
        })?;

        let repo = Arc::new(Self { db, runtime });
        repo.initialize_schema()?;
        Ok(repo)
    }

    fn conn(&self) -> Result<Connection, ModManagerError> {
        self.db
            .connect()
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))
    }

    // TODO: This should be done in a migration
    fn initialize_schema(&self) -> Result<(), ModManagerError> {
        let conn = self.conn()?;
        self.runtime.block_on(async {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS mods (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_registry_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    source_path TEXT,
                    source_kind TEXT,
                    staging_path TEXT,
                    staging_kind TEXT,
                    game_path TEXT,
                    game_kind TEXT,
                    enabler_type TEXT NOT NULL DEFAULT 'symlink',
                    metadata_json TEXT,
                    UNIQUE(game_registry_id, name)
                );
                CREATE TABLE IF NOT EXISTS profiles (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_registry_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    is_active INTEGER NOT NULL DEFAULT 0,
                    UNIQUE(game_registry_id, name)
                );
                CREATE TABLE IF NOT EXISTS profile_mods (
                    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
                    mod_name TEXT NOT NULL,
                    UNIQUE(profile_id, mod_name)
                );",
            )
            .await
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))
        })?;
        Ok(())
    }
}

// TODO: Check if making the operations async is worth it instead of blocking the runtime
impl ModRepository for TursoModRepository {
    fn get_mods(&self, game_registry_id: &str) -> Result<Vec<ReconciledMod>, ModManagerError> {
        let conn = self.conn()?;
        self.runtime.block_on(async {
            let mut rows = conn
                .query(
                    "SELECT name, status, source_path, source_kind, \
                     staging_path, staging_kind, game_path, game_kind, enabler_type, metadata_json \
                     FROM mods WHERE game_registry_id = ?1",
                    params![Value::Text(game_registry_id.to_string())],
                )
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;

            let mut mods = Vec::new();
            while let Some(row) = rows
                .next()
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?
            {
                let name = row_text(&row, 0)?;
                let status = parse_status(&row_text(&row, 1)?);
                let source_entry = entry_from_row(&row, 2, 3, &name);
                let staging_entry = entry_from_row(&row, 4, 5, &name);
                let game_entry = entry_from_row(&row, 6, 7, &name);

                mods.push(ReconciledMod {
                    name,
                    status,
                    source_entry,
                    staging_entry,
                    game_entry,
                    register_id: game_registry_id.to_string(),
                });
            }
            Ok(mods)
        })
    }

    fn upsert_mod(
        &self,
        game_registry_id: &str,
        mod_entry: &ReconciledMod,
    ) -> Result<(), ModManagerError> {
        let conn = self.conn()?;
        let status = Value::Text(status_to_str(mod_entry.status).to_string());
        let (source_path, source_kind) = entry_values(&mod_entry.source_entry);
        let (staging_path, staging_kind) = entry_values(&mod_entry.staging_entry);
        let (game_path, game_kind) = entry_values(&mod_entry.game_entry);

        self.runtime.block_on(async {
            conn.execute(
                "INSERT OR REPLACE INTO mods \
                 (game_registry_id, name, status, \
                  source_path, source_kind, staging_path, staging_kind, \
                  game_path, game_kind, enabler_type) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    Value::Text(game_registry_id.to_string()),
                    Value::Text(mod_entry.name.clone()),
                    status,
                    source_path,
                    source_kind,
                    staging_path,
                    staging_kind,
                    game_path,
                    game_kind,
                    Value::Text(String::from("symlink")),
                ],
            )
            .await
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;
            Ok(())
        })
    }

    fn remove_mod(&self, game_registry_id: &str, name: &str) -> Result<(), ModManagerError> {
        let conn = self.conn()?;
        self.runtime.block_on(async {
            conn.execute(
                "DELETE FROM mods WHERE game_registry_id = ?1 AND name = ?2",
                params![
                    Value::Text(game_registry_id.to_string()),
                    Value::Text(name.to_string())
                ],
            )
            .await
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;
            Ok(())
        })
    }

    fn set_mods(
        &self,
        game_registry_id: &str,
        mods: &[ReconciledMod],
    ) -> Result<(), ModManagerError> {
        let conn = self.conn()?;
        let reg_id = game_registry_id.to_string();
        let batch: Vec<ModRowValues> = mods
            .iter()
            .map(|m| {
                let status = status_to_str(m.status).to_string();
                let (src_path, src_kind) = entry_values(&m.source_entry);
                let (stg_path, stg_kind) = entry_values(&m.staging_entry);
                let (game_path, game_kind) = entry_values(&m.game_entry);
                (
                    m.name.clone(),
                    status,
                    reg_id.clone(),
                    src_path,
                    src_kind,
                    stg_path,
                    stg_kind,
                    game_path,
                    game_kind,
                )
            })
            .collect();

        self.runtime.block_on(async {
            conn.execute("BEGIN TRANSACTION", params![])
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;

            conn.execute(
                "DELETE FROM mods WHERE game_registry_id = ?1",
                params![Value::Text(reg_id.clone())],
            )
            .await
            .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;

            for (
                name,
                status,
                ref rid,
                ref src_p,
                ref src_k,
                ref stg_p,
                ref stg_k,
                ref gm_p,
                ref gm_k,
            ) in &batch
            {
                conn.execute(
                    "INSERT INTO mods (game_registry_id, name, status, source_path, source_kind, \
                     staging_path, staging_kind, game_path, game_kind, enabler_type) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        Value::Text(rid.clone()),
                        Value::Text(name.clone()),
                        Value::Text(status.clone()),
                        src_p.clone(),
                        src_k.clone(),
                        stg_p.clone(),
                        stg_k.clone(),
                        gm_p.clone(),
                        gm_k.clone(),
                        Value::Text(String::from("symlink")),
                    ],
                )
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;
            }

            conn.execute("COMMIT", params![])
                .await
                .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?;

            Ok(())
        })
    }
}

fn row_text(row: &turso::Row, idx: usize) -> Result<String, ModManagerError> {
    match row
        .get_value(idx)
        .map_err(|e| ModManagerError::DatabaseError(e.to_string()))?
    {
        Value::Text(s) => Ok(s),
        Value::Null => Ok(String::new()),
        _ => Err(ModManagerError::DatabaseError(
            "Unexpected non-text value in database".into(),
        )),
    }
}

fn parse_status(s: &str) -> ModStatus {
    match s {
        "Downloaded" => ModStatus::Downloaded,
        "Staged" => ModStatus::Staged,
        "Enabled" => ModStatus::Enabled,
        "Modified" => ModStatus::Modified,
        _ => ModStatus::Downloaded,
    }
}

fn status_to_str(s: ModStatus) -> &'static str {
    match s {
        ModStatus::Downloaded => "Downloaded",
        ModStatus::Staged => "Staged",
        ModStatus::Enabled => "Enabled",
        ModStatus::Modified => "Modified",
    }
}

fn entry_from_row(
    row: &turso::Row,
    path_idx: usize,
    kind_idx: usize,
    name: &str,
) -> Option<ModEntry> {
    let path = row_text(row, path_idx).ok().filter(|s| !s.is_empty())?;
    let kind_str = row_text(row, kind_idx).ok().unwrap_or_default();
    let kind = match kind_str.as_str() {
        "Directory" => ModEntryKind::Directory,
        "ZipArchive" => ModEntryKind::ZipArchive,
        "RarArchive" => ModEntryKind::RarArchive,
        "PakArchive" => ModEntryKind::PakArchive,
        _ => ModEntryKind::Directory,
    };
    Some(ModEntry {
        name: name.to_string(),
        path: path.into(),
        kind,
        metadata: None,
    })
}

fn entry_values(entry: &Option<ModEntry>) -> (Value, Value) {
    match entry {
        Some(e) => {
            let kind_str = match e.kind {
                ModEntryKind::Directory => "Directory",
                ModEntryKind::ZipArchive => "ZipArchive",
                ModEntryKind::RarArchive => "RarArchive",
                ModEntryKind::PakArchive => "PakArchive",
                ModEntryKind::Other => "Other",
            };
            (
                Value::Text(e.path.to_string_lossy().to_string()),
                Value::Text(kind_str.to_string()),
            )
        }
        None => (Value::Null, Value::Null),
    }
}
