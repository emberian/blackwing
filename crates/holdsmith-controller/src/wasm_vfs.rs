//! WASM-compatible in-memory filesystem.
//!
//! This is a simplified version of vfs::MemoryFS that doesn't use SystemTime,
//! making it compatible with WASM environments.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use vfs::error::VfsErrorKind;
use vfs::{FileSystem, SeekAndRead, SeekAndWrite, VfsFileType, VfsMetadata, VfsResult};

type WasmFsHandle = Arc<RwLock<WasmFsImpl>>;

/// A WASM-compatible in-memory filesystem.
///
/// Unlike vfs::MemoryFS, this doesn't call SystemTime::now() which panics in WASM.
/// Instead, it uses a fixed epoch timestamp for all file metadata.
pub struct WasmMemoryFS {
    handle: WasmFsHandle,
}

impl Debug for WasmMemoryFS {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("WASM In-Memory File System")
    }
}

impl WasmMemoryFS {
    /// Create a new WASM-compatible in-memory filesystem.
    pub fn new() -> Self {
        WasmMemoryFS {
            handle: Arc::new(RwLock::new(WasmFsImpl::new())),
        }
    }

    fn ensure_has_parent(&self, path: &str) -> VfsResult<()> {
        let separator = path.rfind('/');
        if let Some(index) = separator {
            if self.exists(&path[..index])? {
                return Ok(());
            }
        }
        Err(VfsErrorKind::Other("Parent path does not exist".into()).into())
    }
}

impl Default for WasmMemoryFS {
    fn default() -> Self {
        Self::new()
    }
}

struct WritableFile {
    content: Cursor<Vec<u8>>,
    destination: String,
    fs: WasmFsHandle,
}

impl Seek for WritableFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.content.seek(pos)
    }
}

impl Write for WritableFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.content.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.content.flush()?;
        let content = self.content.get_ref().clone();
        let mut handle = self.fs.write().unwrap();
        let previous_file = handle.files.get(&self.destination);

        let new_file = WasmFile {
            file_type: VfsFileType::File,
            content: Arc::new(content),
            created: previous_file.map(|file| file.created).unwrap_or(0),
            modified: Some(0),
            accessed: previous_file.and_then(|file| file.accessed),
        };

        handle.files.insert(self.destination.clone(), new_file);
        Ok(())
    }
}

impl Drop for WritableFile {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}


struct ReadableFile {
    content: Arc<Vec<u8>>,
    position: u64,
}

impl ReadableFile {
    fn len(&self) -> u64 {
        self.content.len() as u64 - self.position
    }
}

impl Read for ReadableFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = std::cmp::min(buf.len(), self.len() as usize);
        if amt == 1 {
            buf[0] = self.content[self.position as usize];
        } else {
            buf[..amt].copy_from_slice(
                &self.content.as_slice()[self.position as usize..self.position as usize + amt],
            );
        }
        self.position += amt as u64;
        Ok(amt)
    }
}

impl Seek for ReadableFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.position = offset,
            SeekFrom::Current(offset) => self.position = (self.position as i64 + offset) as u64,
            SeekFrom::End(offset) => self.position = (self.content.len() as i64 + offset) as u64,
        }
        Ok(self.position)
    }
}


impl FileSystem for WasmMemoryFS {
    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        let prefix = format!("{path}/");
        let handle = self.handle.read().unwrap();
        let mut found_directory = false;
        let entries: Vec<_> = handle
            .files
            .iter()
            .filter_map(|(candidate_path, _)| {
                if candidate_path == path {
                    found_directory = true;
                }
                if candidate_path.starts_with(&prefix) {
                    let rest = &candidate_path[prefix.len()..];
                    if !rest.contains('/') {
                        return Some(rest.to_string());
                    }
                }
                None
            })
            .collect();
        if !found_directory {
            return Err(VfsErrorKind::FileNotFound.into());
        }
        Ok(Box::new(entries.into_iter()))
    }

    fn create_dir(&self, path: &str) -> VfsResult<()> {
        self.ensure_has_parent(path)?;
        let map = &mut self.handle.write().unwrap().files;
        let entry = map.entry(path.to_string());
        match entry {
            Entry::Occupied(file) => {
                return match file.get().file_type {
                    VfsFileType::File => Err(VfsErrorKind::FileExists.into()),
                    VfsFileType::Directory => Err(VfsErrorKind::DirectoryExists.into()),
                }
            }
            Entry::Vacant(_) => {
                map.insert(
                    path.to_string(),
                    WasmFile {
                        file_type: VfsFileType::Directory,
                        content: Default::default(),
                        created: 0,
                        modified: Some(0),
                        accessed: Some(0),
                    },
                );
            }
        }
        Ok(())
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        // Skip setting access time (would need SystemTime)
        let handle = self.handle.read().unwrap();
        let file = handle.files.get(path).ok_or(VfsErrorKind::FileNotFound)?;
        ensure_file(file)?;
        Ok(Box::new(ReadableFile {
            content: file.content.clone(),
            position: 0,
        }))
    }

    fn create_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        self.ensure_has_parent(path)?;
        let content = Arc::new(Vec::<u8>::new());
        self.handle.write().unwrap().files.insert(
            path.to_string(),
            WasmFile {
                file_type: VfsFileType::File,
                content,
                created: 0,
                modified: Some(0),
                accessed: Some(0),
            },
        );
        let writer = WritableFile {
            content: Cursor::new(vec![]),
            destination: path.to_string(),
            fs: self.handle.clone(),
        };
        Ok(Box::new(writer))
    }

    fn append_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        let handle = self.handle.write().unwrap();
        let file = handle.files.get(path).ok_or(VfsErrorKind::FileNotFound)?;
        let mut content = Cursor::new(file.content.as_ref().clone());
        content.seek(SeekFrom::End(0))?;
        let writer = WritableFile {
            content,
            destination: path.to_string(),
            fs: self.handle.clone(),
        };
        Ok(Box::new(writer))
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        let guard = self.handle.read().unwrap();
        let files = &guard.files;
        let file = files.get(path).ok_or(VfsErrorKind::FileNotFound)?;
        Ok(VfsMetadata {
            file_type: file.file_type,
            len: file.content.len() as u64,
            // Use UNIX epoch as a stub timestamp
            modified: file.modified.map(|_| SystemTime::UNIX_EPOCH),
            created: Some(SystemTime::UNIX_EPOCH),
            accessed: file.accessed.map(|_| SystemTime::UNIX_EPOCH),
        })
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        Ok(self.handle.read().unwrap().files.contains_key(path))
    }

    fn remove_file(&self, path: &str) -> VfsResult<()> {
        let mut handle = self.handle.write().unwrap();
        handle
            .files
            .remove(path)
            .ok_or(VfsErrorKind::FileNotFound)?;
        Ok(())
    }

    fn remove_dir(&self, path: &str) -> VfsResult<()> {
        if self.read_dir(path)?.next().is_some() {
            return Err(VfsErrorKind::Other("Directory to remove is not empty".into()).into());
        }
        let mut handle = self.handle.write().unwrap();
        handle
            .files
            .remove(path)
            .ok_or(VfsErrorKind::FileNotFound)?;
        Ok(())
    }
}

struct WasmFsImpl {
    files: HashMap<String, WasmFile>,
}

impl WasmFsImpl {
    pub fn new() -> Self {
        let mut files = HashMap::new();
        // Add root directory
        files.insert(
            "".to_string(),
            WasmFile {
                file_type: VfsFileType::Directory,
                content: Arc::new(vec![]),
                created: 0,
                modified: None,
                accessed: None,
            },
        );
        Self { files }
    }
}

struct WasmFile {
    file_type: VfsFileType,
    content: Arc<Vec<u8>>,
    // Use u64 timestamps instead of SystemTime (milliseconds since epoch, or just 0)
    created: u64,
    modified: Option<u64>,
    accessed: Option<u64>,
}

fn ensure_file(file: &WasmFile) -> VfsResult<()> {
    if file.file_type != VfsFileType::File {
        return Err(VfsErrorKind::Other("Not a file".into()).into());
    }
    Ok(())
}
