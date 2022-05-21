use crate::Error;
use md5::Context;
use std::fs;
use std::fs::{File, Permissions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Store {
    basedir: PathBuf,
    baseurl: String,
}

impl Store {
    pub fn new(basedir: impl AsRef<Path>, baseurl: &str) -> Self {
        Store {
            basedir: basedir.as_ref().to_path_buf(),
            baseurl: baseurl.trim_end_matches("/").to_string(),
        }
    }

    pub async fn create(&self, name: &str) -> Result<File, Error> {
        let path = self.generate_path(name);
        fs::create_dir_all(path.parent().unwrap())?;

        let file = File::create(&path)?;
        file.set_permissions(Permissions::from_mode(0o644))?;

        Ok(file)
    }

    pub fn remove(&self, name: &str) -> std::io::Result<()> {
        fs::remove_file(self.generate_path(name))
    }

    pub fn hash(&self, name: &str) -> Result<[u8; 16], Error> {
        let path = self.generate_path(name);
        let mut file = File::open(path)?;

        let mut hash = Context::new();

        let mut buff = vec![0; 1024 * 1024];

        loop {
            let read = file.read(&mut buff)?;

            if read == 0 {
                break;
            }

            let data = &buff[0..read];
            hash.consume(&data);
        }

        Ok(hash.compute().0)
    }

    pub fn generate_path(&self, name: &str) -> PathBuf {
        let mut path = self.basedir.clone();
        path.push(&name[0..2]);
        path.push(&name[2..4]);
        path.push(name);
        path
    }
}
