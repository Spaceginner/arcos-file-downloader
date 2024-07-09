use std::path::{Path, PathBuf};
use gumdrop::Options;
use crate::server::{Server, FS};

mod options;
mod server;


#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let opts = options::AppOptions::parse_args_default_or_exit();

    // --- init
    let (server, user, fs);
    {
        let initialising_span = tracing::info_span!("init");
        let _is_guard = initialising_span.enter();

        server = Server::new(opts.api, opts.code.as_deref()).await.unwrap();

        user = server.auth_user(&opts.username, &opts.password).await.unwrap();

        tracing::info!("authenticated");

        fs = user.fs();
    };

    // --- downloading
    {
        let downloading_span = tracing::span!(tracing::Level::INFO, "download_files");
        let _ds_guard = downloading_span.enter();

        let files = fs.tree().await.unwrap();

        tracing::info!("found {} file(s)", files.len());

        futures::future::join_all(
            files.into_iter().enumerate().map(|(id, p)| download_file(&fs, id+1, &opts.out, p))
        ).await;

        tracing::info!("downloaded all files");
    };
}


async fn download_file(fs: &FS<'_>, id: usize, out_path: &Path, path: PathBuf) {
    let logged_path = path.to_str().unwrap();
    tracing::info!("downloading #{id} ({logged_path})...");

    let file_data = fs.read(&path).await.unwrap();

    let final_out_path = out_path.join(&path);

    if let Some(parent) = final_out_path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        };
    };

    tokio::fs::write(final_out_path, file_data).await.unwrap();

    tracing::info!("downloaded #{id} ({logged_path})");
}
