use crate::config::{Config, ConfigError};
use clap::Parser;
use demostf_client::{ApiClient, Demo, ListOrder, ListParams};
use main_error::{MainError, MainResult};
use md5::Context;
use secretfile::{load, SecretError};
use std::fs::{copy, create_dir_all, remove_file, write, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::time::timeout;
use tracing::{error, info, instrument, warn};

mod config;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Request(#[from] std::io::Error),
    #[error(transparent)]
    Api(#[from] demostf_client::Error),
    #[error("Backup timed out")]
    Timeout,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Secret(#[from] SecretError),
}

#[derive(Debug, Parser)]
struct Args {
    /// Config file
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let config = Config::load(args.config)?;

    let source_root: PathBuf = config.source.root.trim_end_matches('/').into();
    let target_root: PathBuf = config.target.root.trim_end_matches('/').into();
    let api_key = load(&config.api.key_file)?;
    let source_backend = config.source.backend;
    let target_backend = config.target.backend;
    let age = config.migrate.age;

    let cutoff = OffsetDateTime::now_utc() - Duration::from_secs(age);
    info!(cutoff = display(cutoff), "starting move");

    let client = ApiClient::new();

    let demos = client
        .list(
            ListParams::default()
                .with_before(cutoff)
                .with_order(ListOrder::Ascending)
                .with_backend(source_backend),
            1,
        )
        .await?;

    for demo in demos {
        let name = demo.path.rsplit('/').next().unwrap();

        let source_path = generate_path(&source_root, name);
        let target_path = generate_path(&target_root, name);

        move_demo(
            &client,
            &demo,
            source_path,
            target_path,
            &target_backend,
            &api_key,
        )
        .await?;
    }

    Ok(())
}

#[instrument(skip_all, fields(demo = demo.id, target_path = display(target_path.display()), source_path = display(source_path.display())))]
async fn move_demo(
    client: &ApiClient,
    demo: &Demo,
    source_path: PathBuf,
    target_path: PathBuf,
    target_backend: &str,
    api_key: &str,
) -> MainResult {
    if !source_path.is_file() {
        warn!("source not found, re-downloading");
        re_download(client, &target_path, demo).await?;
    }
    if target_path.is_file() {
        warn!("target exists");
    } else {
        create_dir_all(target_path.parent().unwrap())?;

        copy(&source_path, &target_path)?;
    }

    let calculated_hash = hash(&target_path)?;

    if calculated_hash != demo.hash {
        warn!(
            calculated = debug(calculated_hash),
            stored = debug(demo.hash),
            "hash mismatch for target"
        );
    }

    info!("renamed");
    if let Err(err) = client
        .set_url(
            demo.id,
            target_backend,
            &demo.path,
            &demo.url,
            demo.hash,
            api_key,
        )
        .await
    {
        error!(error = display(&err), "error while setting url");
        remove_file(&target_path)?;
        return Err(err.into());
    }
    remove_file(source_path)?;
    Ok(())
}

fn generate_path(basedir: &Path, name: &str) -> PathBuf {
    let mut path = basedir.to_path_buf();
    path.push(&name[0..2]);
    path.push(&name[2..4]);
    path.push(name);
    path
}

fn hash<P: AsRef<Path>>(path: P) -> Result<[u8; 16], Error> {
    let mut file = File::open(path)?;

    let mut hash = Context::new();

    let mut buff = vec![0; 1024 * 1024];

    loop {
        let read = file.read(&mut buff)?;

        if read == 0 {
            break;
        }

        let data = &buff[0..read];
        hash.consume(data);
    }

    Ok(hash.compute().0)
}

#[instrument(skip(demo), fields(id = demo.id, target = display(target.display())))]
async fn re_download(client: &ApiClient, target: &Path, demo: &Demo) -> Result<(), Error> {
    let mut data = Vec::with_capacity(demo.duration as usize / 60 * 1024);

    timeout(Duration::from_secs(5 * 60), demo.save(client, &mut data))
        .await
        .map_err(|_| Error::Timeout)??;

    write(target, data)?;

    Ok(())
}
