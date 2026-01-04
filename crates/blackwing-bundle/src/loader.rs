//! Bundle loading from directories and archives.

use crate::bundle::{GameBundle, ScriptSource};
use crate::content::{DialogueTree, EntityTemplate, QuestDef};
use crate::error::BundleError;
use crate::manifest::BundleManifest;
use crate::schema::BundleSchema;
use blackwing_tilemap::{Room, TileSet, WorldMap};
use smol_str::SmolStr;
use std::path::Path;

/// Loader for game bundles.
///
/// Supports loading from:
/// - Directories (for development)
/// - Zip archives (.bwg files, when `archive` feature is enabled)
pub struct BundleLoader;

impl BundleLoader {
    /// Load a bundle from a directory.
    ///
    /// Expected structure:
    /// ```text
    /// bundle_dir/
    /// ├── manifest.toml
    /// ├── schema.toml (optional)
    /// ├── scripts/
    /// │   ├── systems/
    /// │   │   └── *.rhai
    /// │   └── init.rhai (optional)
    /// └── content/
    ///     ├── entities/
    ///     │   └── *.toml
    ///     ├── dialogues/
    ///     │   └── *.toml
    ///     └── quests/
    ///         └── *.toml
    /// ```
    pub fn load_from_dir(path: &Path) -> Result<GameBundle, BundleError> {
        // Load manifest
        let manifest_path = path.join("manifest.toml");
        let manifest_str = std::fs::read_to_string(&manifest_path)?;
        let manifest: BundleManifest = toml::from_str(&manifest_str)?;

        // Load schema (optional)
        let schema_path = path.join("schema.toml");
        let schema = if schema_path.exists() {
            let schema_str = std::fs::read_to_string(&schema_path)?;
            toml::from_str(&schema_str)?
        } else {
            BundleSchema::default()
        };

        // Load script systems
        let script_systems = Self::load_script_systems(path)?;

        // Load init script
        let init_script = Self::load_init_script(path)?;

        // Load content
        let templates = Self::load_templates(path)?;
        let dialogues = Self::load_dialogues(path)?;
        let quests = Self::load_quests(path)?;

        // Load tilemap content
        let tilesets = Self::load_tilesets(path)?;
        let (worlds, starting_world) = Self::load_worlds(path)?;

        Ok(GameBundle {
            manifest,
            schema,
            templates,
            dialogues,
            quests,
            script_systems,
            init_script,
            tilesets,
            worlds,
            starting_world,
        })
    }

    /// Load script systems from scripts/systems/*.rhai
    fn load_script_systems(base_path: &Path) -> Result<Vec<ScriptSource>, BundleError> {
        let systems_dir = base_path.join("scripts").join("systems");
        if !systems_dir.exists() {
            return Ok(Vec::new());
        }

        let mut scripts = Vec::new();
        for entry in std::fs::read_dir(&systems_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                let source = std::fs::read_to_string(&path)?;
                scripts.push(ScriptSource {
                    id: SmolStr::new(id),
                    source,
                });
            }
        }

        Ok(scripts)
    }

    /// Load init script from scripts/init.rhai
    fn load_init_script(base_path: &Path) -> Result<Option<String>, BundleError> {
        let init_path = base_path.join("scripts").join("init.rhai");
        if init_path.exists() {
            Ok(Some(std::fs::read_to_string(&init_path)?))
        } else {
            Ok(None)
        }
    }

    /// Load entity templates from content/entities/*.toml
    fn load_templates(base_path: &Path) -> Result<Vec<EntityTemplate>, BundleError> {
        let entities_dir = base_path.join("content").join("entities");
        if !entities_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();
        for entry in std::fs::read_dir(&entities_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = std::fs::read_to_string(&path)?;

                // Try to parse as a single template or a list of templates
                if let Ok(template) = toml::from_str::<EntityTemplate>(&content) {
                    templates.push(template);
                } else if let Ok(list) = toml::from_str::<TemplateList>(&content) {
                    templates.extend(list.templates);
                }
            }
        }

        Ok(templates)
    }

    /// Load dialogues from content/dialogues/*.toml
    fn load_dialogues(base_path: &Path) -> Result<Vec<DialogueTree>, BundleError> {
        let dialogues_dir = base_path.join("content").join("dialogues");
        if !dialogues_dir.exists() {
            return Ok(Vec::new());
        }

        let mut dialogues = Vec::new();
        for entry in std::fs::read_dir(&dialogues_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = std::fs::read_to_string(&path)?;

                if let Ok(dialogue) = toml::from_str::<DialogueTree>(&content) {
                    dialogues.push(dialogue);
                } else if let Ok(list) = toml::from_str::<DialogueList>(&content) {
                    dialogues.extend(list.dialogues);
                }
            }
        }

        Ok(dialogues)
    }

    /// Load quests from content/quests/*.toml
    fn load_quests(base_path: &Path) -> Result<Vec<QuestDef>, BundleError> {
        let quests_dir = base_path.join("content").join("quests");
        if !quests_dir.exists() {
            return Ok(Vec::new());
        }

        let mut quests = Vec::new();
        for entry in std::fs::read_dir(&quests_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = std::fs::read_to_string(&path)?;

                if let Ok(quest) = toml::from_str::<QuestDef>(&content) {
                    quests.push(quest);
                } else if let Ok(list) = toml::from_str::<QuestList>(&content) {
                    quests.extend(list.quests);
                }
            }
        }

        Ok(quests)
    }

    /// Load tilesets from tilesets/*.toml
    fn load_tilesets(base_path: &Path) -> Result<Vec<TileSet>, BundleError> {
        let tilesets_dir = base_path.join("tilesets");
        if !tilesets_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tilesets = Vec::new();
        for entry in std::fs::read_dir(&tilesets_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = std::fs::read_to_string(&path)?;

                if let Ok(tileset) = toml::from_str::<TileSet>(&content) {
                    tilesets.push(tileset);
                } else if let Ok(list) = toml::from_str::<TileSetList>(&content) {
                    tilesets.extend(list.tilesets);
                }
            }
        }

        Ok(tilesets)
    }

    /// Load world maps from worlds/*/world.toml
    fn load_worlds(base_path: &Path) -> Result<(Vec<WorldMap>, Option<SmolStr>), BundleError> {
        let worlds_dir = base_path.join("worlds");
        if !worlds_dir.exists() {
            return Ok((Vec::new(), None));
        }

        let mut worlds = Vec::new();
        let mut starting_world = None;

        // Check for world index file
        let index_path = worlds_dir.join("index.toml");
        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            if let Ok(index) = toml::from_str::<WorldIndex>(&content) {
                starting_world = index.starting_world;
            }
        }

        // Load each world directory
        for entry in std::fs::read_dir(&worlds_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let world_toml = path.join("world.toml");
                if world_toml.exists() {
                    let content = std::fs::read_to_string(&world_toml)?;
                    let mut world: WorldMap = toml::from_str(&content)?;

                    // Load rooms from rooms/*.toml
                    let rooms_dir = path.join("rooms");
                    if rooms_dir.exists() {
                        for room_entry in std::fs::read_dir(&rooms_dir)? {
                            let room_entry = room_entry?;
                            let room_path = room_entry.path();

                            if room_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                                let room_content = std::fs::read_to_string(&room_path)?;
                                let room: Room = toml::from_str(&room_content)?;
                                world.rooms.insert(room.id.clone(), room);
                            }
                        }
                    }

                    // If no starting world set, use first one found
                    if starting_world.is_none() && !worlds.is_empty() == false {
                        starting_world = Some(world.id.clone());
                    }

                    worlds.push(world);
                }
            }
        }

        Ok((worlds, starting_world))
    }

    /// Save a bundle to a directory.
    pub fn save_to_dir(bundle: &GameBundle, path: &Path) -> Result<(), BundleError> {
        // Create directories
        std::fs::create_dir_all(path)?;
        std::fs::create_dir_all(path.join("scripts").join("systems"))?;
        std::fs::create_dir_all(path.join("content").join("entities"))?;
        std::fs::create_dir_all(path.join("content").join("dialogues"))?;
        std::fs::create_dir_all(path.join("content").join("quests"))?;

        // Save manifest
        let manifest_toml = toml::to_string_pretty(&bundle.manifest)?;
        std::fs::write(path.join("manifest.toml"), manifest_toml)?;

        // Save schema
        let schema_toml = toml::to_string_pretty(&bundle.schema)?;
        std::fs::write(path.join("schema.toml"), schema_toml)?;

        // Save script systems
        for script in &bundle.script_systems {
            let script_path = path
                .join("scripts")
                .join("systems")
                .join(format!("{}.rhai", script.id));
            std::fs::write(script_path, &script.source)?;
        }

        // Save init script
        if let Some(init) = &bundle.init_script {
            std::fs::write(path.join("scripts").join("init.rhai"), init)?;
        }

        // Save templates (grouped by kind)
        let templates_path = path.join("content").join("entities").join("all.toml");
        let templates_toml = toml::to_string_pretty(&TemplateList {
            templates: bundle.templates.clone(),
        })?;
        std::fs::write(templates_path, templates_toml)?;

        // Save dialogues
        let dialogues_path = path.join("content").join("dialogues").join("all.toml");
        let dialogues_toml = toml::to_string_pretty(&DialogueList {
            dialogues: bundle.dialogues.clone(),
        })?;
        std::fs::write(dialogues_path, dialogues_toml)?;

        // Save quests
        let quests_path = path.join("content").join("quests").join("all.toml");
        let quests_toml = toml::to_string_pretty(&QuestList {
            quests: bundle.quests.clone(),
        })?;
        std::fs::write(quests_path, quests_toml)?;

        Ok(())
    }

    /// Load a bundle from a zip archive.
    #[cfg(feature = "archive")]
    pub fn load_from_archive(path: &Path) -> Result<GameBundle, BundleError> {
        use std::io::Read;

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| BundleError::Archive(format!("Failed to open archive: {}", e)))?;

        // Read manifest
        let manifest: BundleManifest = {
            let mut file = archive
                .by_name("manifest.toml")
                .map_err(|e| BundleError::NotFound(format!("manifest.toml: {}", e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            toml::from_str(&content)?
        };

        // Read schema (optional)
        let schema: BundleSchema = match archive.by_name("schema.toml") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                toml::from_str(&content)?
            }
            Err(_) => BundleSchema::default(),
        };

        // Read script systems
        let mut script_systems = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.starts_with("scripts/systems/") && name.ends_with(".rhai") {
                let id = name
                    .strip_prefix("scripts/systems/")
                    .and_then(|s| s.strip_suffix(".rhai"))
                    .unwrap_or("unknown");

                let mut source = String::new();
                file.read_to_string(&mut source)?;

                script_systems.push(ScriptSource {
                    id: SmolStr::new(id),
                    source,
                });
            }
        }

        // Read init script
        let init_script = match archive.by_name("scripts/init.rhai") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                Some(content)
            }
            Err(_) => None,
        };

        // For simplicity, we expect content to be serialized in archive
        // A real implementation would parse individual TOML files

        Ok(GameBundle {
            manifest,
            schema,
            templates: Vec::new(),
            dialogues: Vec::new(),
            quests: Vec::new(),
            script_systems,
            init_script,
            tilesets: Vec::new(),
            worlds: Vec::new(),
            starting_world: None,
        })
    }

    /// Save a bundle to a zip archive.
    #[cfg(feature = "archive")]
    pub fn save_to_archive(bundle: &GameBundle, path: &Path) -> Result<(), BundleError> {
        use std::io::Write;
        use zip::write::SimpleFileOptions;

        let file = std::fs::File::create(path)?;
        let mut archive = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Write manifest
        archive.start_file("manifest.toml", options)
            .map_err(|e| BundleError::Archive(e.to_string()))?;
        let manifest_toml = toml::to_string_pretty(&bundle.manifest)?;
        archive.write_all(manifest_toml.as_bytes())?;

        // Write schema
        archive.start_file("schema.toml", options)
            .map_err(|e| BundleError::Archive(e.to_string()))?;
        let schema_toml = toml::to_string_pretty(&bundle.schema)?;
        archive.write_all(schema_toml.as_bytes())?;

        // Write script systems
        for script in &bundle.script_systems {
            let name = format!("scripts/systems/{}.rhai", script.id);
            archive.start_file(&name, options)
                .map_err(|e| BundleError::Archive(e.to_string()))?;
            archive.write_all(script.source.as_bytes())?;
        }

        // Write init script
        if let Some(init) = &bundle.init_script {
            archive.start_file("scripts/init.rhai", options)
                .map_err(|e| BundleError::Archive(e.to_string()))?;
            archive.write_all(init.as_bytes())?;
        }

        // Write content as serialized TOML
        archive.start_file("content/entities.toml", options)
            .map_err(|e| BundleError::Archive(e.to_string()))?;
        let templates_toml = toml::to_string_pretty(&TemplateList {
            templates: bundle.templates.clone(),
        })?;
        archive.write_all(templates_toml.as_bytes())?;

        archive.start_file("content/dialogues.toml", options)
            .map_err(|e| BundleError::Archive(e.to_string()))?;
        let dialogues_toml = toml::to_string_pretty(&DialogueList {
            dialogues: bundle.dialogues.clone(),
        })?;
        archive.write_all(dialogues_toml.as_bytes())?;

        archive.start_file("content/quests.toml", options)
            .map_err(|e| BundleError::Archive(e.to_string()))?;
        let quests_toml = toml::to_string_pretty(&QuestList {
            quests: bundle.quests.clone(),
        })?;
        archive.write_all(quests_toml.as_bytes())?;

        archive.finish()
            .map_err(|e| BundleError::Archive(e.to_string()))?;

        Ok(())
    }
}

// Helper structs for TOML list parsing
#[derive(serde::Deserialize, serde::Serialize)]
struct TemplateList {
    #[serde(default)]
    templates: Vec<EntityTemplate>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct DialogueList {
    #[serde(default)]
    dialogues: Vec<DialogueTree>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct QuestList {
    #[serde(default)]
    quests: Vec<QuestDef>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct TileSetList {
    #[serde(default)]
    tilesets: Vec<TileSet>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WorldIndex {
    #[serde(default)]
    starting_world: Option<SmolStr>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_bundle() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();

        // Create a test bundle
        let mut bundle = GameBundle::new("Test Game");
        bundle.manifest.author = Some("Test Author".to_string());
        bundle.add_script_system(
            "weather",
            r#"
const SYSTEM_ID = "weather";
const HANDLES = ["set_weather"];

fn handle_command(cmd) {
    chronicle("Weather", "Changed weather");
}
"#,
        );
        bundle.set_init_script("// Init script\nset_resource('gold', 100);");

        // Save
        BundleLoader::save_to_dir(&bundle, path).unwrap();

        // Verify files exist
        assert!(path.join("manifest.toml").exists());
        assert!(path.join("schema.toml").exists());
        assert!(path.join("scripts/systems/weather.rhai").exists());
        assert!(path.join("scripts/init.rhai").exists());

        // Load
        let loaded = BundleLoader::load_from_dir(path).unwrap();
        assert_eq!(loaded.manifest.name, "Test Game");
        assert_eq!(loaded.manifest.author.as_deref(), Some("Test Author"));
        assert_eq!(loaded.script_systems.len(), 1);
        assert!(loaded.init_script.is_some());
    }

    #[cfg(feature = "archive")]
    #[test]
    fn save_and_load_archive() {
        let tmp = TempDir::new().unwrap();
        let archive_path = tmp.path().join("test.bwg");

        // Create a test bundle
        let mut bundle = GameBundle::new("Archive Test");
        bundle.add_script_system(
            "combat",
            r#"
const SYSTEM_ID = "combat";
const HANDLES = ["attack"];

fn handle_command(cmd) {
    chronicle("Combat", "Attack!");
}
"#,
        );

        // Save to archive
        BundleLoader::save_to_archive(&bundle, &archive_path).unwrap();
        assert!(archive_path.exists());

        // Load from archive
        let loaded = BundleLoader::load_from_archive(&archive_path).unwrap();
        assert_eq!(loaded.manifest.name, "Archive Test");
        assert_eq!(loaded.script_systems.len(), 1);
    }
}
