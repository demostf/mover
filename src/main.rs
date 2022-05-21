use demostf_client::{ApiClient, ListOrder, ListParams};
use main_error::MainError;
use md5::Context;
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, remove_file, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use time::OffsetDateTime;
use tracing::{error, info, info_span};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Request(#[from] std::io::Error),
    #[error(transparent)]
    Api(#[from] demostf_client::Error),
    #[error("Backup timed out")]
    Timeout,
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt::init();

    let mut args: HashMap<_, _> = dotenv::vars().collect();

    let source_root: PathBuf = args
        .remove("SOURCE_ROOT")
        .expect("no SOURCE_ROOT set")
        .trim_end_matches("/")
        .into();
    let target_root: PathBuf = args
        .remove("TARGET_ROOT")
        .expect("no TARGET_ROOT set")
        .trim_end_matches("/")
        .into();
    let api_key = args.remove("KEY").expect("no KEY set");
    let source_backend = args
        .remove("SOURCE_BACKEND")
        .expect("no SOURCE_BACKEND set");
    let target_backend = args
        .remove("TARGET_BACKEND")
        .expect("no TARGET_BACKEND set");
    let age: u64 = args
        .get("AGE")
        .expect("no AGE set")
        .parse()
        .expect("invalid AGE");

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
        let _span = info_span!(
            "name",
            demo = demo.id,
            source_path = display(source_path.display()),
            target_path = display(target_path.display()),
        )
        .entered();

        if !source_path.is_file() {
            error!("source not found");
            return Ok(());
        }
        if target_path.is_file() {
            error!("target exists");
            return Ok(());
        }

        let calculated_hash = hash(&source_path)?;
        if calculated_hash != demo.hash {
            error!(
                calculated = debug(calculated_hash),
                stored = debug(demo.hash),
                "hash mismatch"
            );
            return Ok(());
        }

        create_dir_all(target_path.parent().unwrap())?;

        copy(&source_path, &target_path)?;

        let calculated_hash = hash(&target_path)?;

        if calculated_hash != demo.hash {
            error!(
                calculated = debug(calculated_hash),
                stored = debug(demo.hash),
                "hash mismatch for target"
            );
            remove_file(&target_path)?;
            return Ok(());
        }

        info!("renamed");
        if let Err(err) = client
            .set_url(
                demo.id,
                &target_backend,
                &demo.path,
                &demo.url,
                calculated_hash,
                &api_key,
            )
            .await
        {
            error!(error = display(&err), "error while setting url");
            remove_file(&target_path)?;
            return Err(err.into());
        }
        remove_file(source_path)?;
    }

    Ok(())
}

fn generate_path(basedir: &PathBuf, name: &str) -> PathBuf {
    let mut path = basedir.clone();
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
        hash.consume(&data);
    }

    Ok(hash.compute().0)
}
