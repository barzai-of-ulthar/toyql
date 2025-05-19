/// Utilities for storing data to the filesystem and retrieving it thence.

use std::{hash::Hash, result::Result};
use tempfile::{TempDir, tempdir};


#[allow(dead_code)]  // TODO!
/// A key into stored data.
pub type StorageToken = String;

#[allow(dead_code)]  // TODO!
/// Location abstraction
/// 
/// This enum specifies different storage scopes, which will map to different storage
/// locations.
pub enum StorageScope {
    /// Objects stored at this scope may become unavailable as soon as associated store object
    /// is dropped.  This will typically mean `$TMPDIR`` storage with an attempt to clean up
    /// at or near object dtor time.
    ///
    /// NOTE:  There is no sound Rust or POSIX way to guarantee end-of-process file deletion.
    /// Deletion of `Objects`-scoped files is best-effort and will often fail.  This is not a
    /// security feature.
    Object,

    /// Objects stored at this scope use system temporary storage (`$TMPDIR`), and no attempt
    /// is made to delete them at object destruction or process exit; this is desirable for
    /// debugging.
    Temporary,

    /// Objects stored at this scope will remain available to the current user so long as
    /// this user or superusers do not interfere.  This will typically mean storage in
    /// `$HOME` or under `/var`.
    User,
}

/// Atomic, key-value string store.
///
/// This stores strings in a scope that allows them to be retrieved by an associated key.
/// 
/// The store is _conceptually_ mutable:  Even though writing to the store may not change the
/// in-memory content of the structure, it is a mutation of the store for data integrity
/// purposes and so callers are required to observe Rust's read-write lock semantics.
pub struct AtomicKVStringStore {
    scope: StorageScope,
    directory_path: std::path::PathBuf,
    scope_holder: Option<TempDir>,
}

#[allow(dead_code)]  // TODO!
impl AtomicKVStringStore {
    /// Create a new store.  Out-of-process objects (files and directories) will where
    /// practical contain `hint` in some prominent place, for debugging purposes.
    pub fn new(scope: StorageScope, hint: &str)
            -> Result<AtomicKVStringStore, std::io::Error> {
        match scope {
            StorageScope::Object | StorageScope::Temporary => {
                let temp_holder: TempDir = tempdir()?;
                let temp_dir_base = temp_holder.path().to_path_buf();
                let temp_dir_path = temp_dir_base.join("toyql_".to_string() + hint);
                std::fs::create_dir(&temp_dir_path)?;
                Ok(AtomicKVStringStore{
                    scope,
                    scope_holder: Some(temp_holder),
                    directory_path: temp_dir_path,
                })
            },
            StorageScope::User => {
                // Get this from some broader config or environment variable.
                let cache_parent = std::env::var("HOME").unwrap_or("/run".to_string());
                let cache_dir = std::path::Path::new(&cache_parent).join(".cache/toyql");
                Ok(AtomicKVStringStore{
                    scope,
                    scope_holder: None,
                    directory_path: cache_dir,
                })
            },
        }
    }

    /// A key may not be suitable for use as a filesystem name; for instance, it may be very
    /// long or contain special characters.  However we prefer to use the key when possible
    /// to simplify debugging.  This function returns a key suitable for storage.
    fn filename_for_key(key: &str) -> String {
        let valid: bool = key.len() < 32
                            && !key.contains(|c: char| {
                                !c.is_ascii_alphanumeric() &&
                                !"_-".contains(c)});
        if valid {
            format!("literal_key_{}", key)
        } else {
            let mut hasher = std::hash::DefaultHasher::new();
            key.hash(&mut hasher);
            format!("_hashed_key_{:x}", std::hash::Hasher::finish(&hasher))
        }
    }

    fn path_for_key(&self, key: &str) -> std::path::PathBuf {
        let key_filename = AtomicKVStringStore::filename_for_key(key);
        self.directory_path.join(key_filename)
    }

    pub fn store(&mut self, key: &StorageToken, content: &str) -> Result<StorageToken, String> {
        // No simple procedure will atomically change a file in all cases (cf `fuse`
        // filesystems), but writing and then renaming works on all posix-compliant file
        // systems because the rename system call at the base of `rename()` says:
        //
        //     The rename() system call guarantees that an instance of new will always exist,
        //     even if the system should crash in the middle of the operation.
        let key_filename = AtomicKVStringStore::filename_for_key(key);
        let tmp_path = self.directory_path.join(key_filename.clone() + "_tmp");
        let target_path = self.directory_path.join(&key_filename);
        std::fs::write(&tmp_path,
                        content.as_bytes()).map_err(
                            |_| format!("Temp file {} could not be written", tmp_path.to_str().unwrap()))?;
        std::fs::rename(&tmp_path, &target_path).map_err(
                            |_| format!("Temp file {} could not be moved to {}",
                                            tmp_path.to_str().unwrap(),
                                            target_path.to_str().unwrap()))?;
        Ok(key.to_string())
    }

    pub fn get(&self, key: &StorageToken) -> Result<String, String> {
        let path = self.path_for_key(key);
        std::fs::read_to_string(&path).map_err(
            |_| format!("Storage file {} could not be read", path.to_str().unwrap()))
    }

    pub fn count(&self) -> usize {
        let contents = std::fs::read_dir(&self.directory_path).unwrap();
        contents.count()
    }

    pub fn del(&mut self, key: &StorageToken) -> bool {
        let path = self.path_for_key(key);
        match std::fs::remove_file(path) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl Drop for AtomicKVStringStore {
    fn drop(&mut self) {
        if let StorageScope::Object = self.scope {
            self.scope_holder = None;
            std::fs::remove_dir_all(&self.directory_path).unwrap_or_default();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        // Once through the happy path of the API.
        let mut dut = AtomicKVStringStore::new(StorageScope::Temporary, "smoke_test").unwrap();
        assert_eq!(dut.count(), 0);
        let key = dut.store(&"foo".to_string(), "foo_content").unwrap();
        assert_eq!(key, "foo");
        assert_eq!(dut.count(), 1);
        let value = dut.get(&key).unwrap();
        assert_eq!(value, "foo_content");
        assert!(dut.del(&key));
        assert_eq!(dut.count(), 0);
    }
}
