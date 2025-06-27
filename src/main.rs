use async_walkdir::WalkDir;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use futures::TryStreamExt;
use trauma::download::Download;
use trauma::downloader::DownloaderBuilder;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Dummy flag for compatibility as ChRIS plugin. Does nothing.
    #[clap(long = "/dummy_selfexec", hide = true)]
    dummy_selfexec: bool,

    /// Required useless ChRIS plugin flag. Does nothing.
    #[clap(long, hide = true)]
    saveinputmeta: bool,

    /// Required useless ChRIS plugin flag. Does nothing.
    #[clap(long, hide = true)]
    saveoutputmeta: bool,

    /// Number of retries per download
    #[clap(short, long, default_value_t = 3)]
    retries: u32,

    /// Maximum number of concurrent downloads
    #[clap(short = 'J', long, default_value_t = 32)]
    concurrency: usize,

    /// URL to download. If specified, activate "single" mode.
    #[clap(short, long, group = "input")]
    url: Option<url::Url>,

    /// Data directory. In "bulk" mode, this directory is treated as an
    /// input directory containing text files of URLs separated by whitespace.
    /// In "single" mode, it is the directory where downloaded outputs
    /// are saved to.
    #[clap()]
    dir: Utf8PathBuf,

    /// Output directory, only used in "bulk" download mode.
    #[clap(group = "input")]
    output_dir: Option<Utf8PathBuf>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();

    let downloads = if let Some(url) = &args.url {
        vec![Download::try_from(url)?]
    } else {
        read_urls(&args.dir).await?
    };

    let output_dir = args.output_dir.unwrap_or_else(|| args.dir);

    let downloader = DownloaderBuilder::new()
        .directory(output_dir.into_std_path_buf())
        .retries(args.retries)
        .concurrent_downloads(args.concurrency)
        .build();
    let summaries = downloader.download(&downloads).await;

    let errors: Vec<_> = summaries
        .iter()
        .filter_map(|summary| {
            if let trauma::download::Status::Fail(reason) = summary.status() {
                let url = &summary.download().url;
                let err = color_eyre::eyre::eyre!("Failed to download {url}: {reason}");
                Some(err)
            } else {
                None
            }
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        for error in errors {
            println!("{error}");
        }
        Err(color_eyre::eyre::eyre!("Some downloads failed."))
    }
}

/// Read URLs from all text files found in a directory.
async fn read_urls(input_dir: &Utf8Path) -> color_eyre::Result<Vec<Download>> {
    let file_contents: Vec<(Utf8PathBuf, String)> = WalkDir::new(input_dir)
        .map_err(color_eyre::Report::new)
        .try_filter_map(async |entry| {
            let entry_type = entry.file_type().await.map_err(color_eyre::Report::new)?;
            if entry_type.is_file() {
                let path = Utf8PathBuf::from_path_buf(entry.path())
                    .map_err(|p| color_eyre::eyre::eyre!("Path contains non-UTF8: {p:?}"))?;
                if let Some(name) = path.file_name() && matches!(name, "input.meta.json" | "output.meta.json") {
                    return Ok(None);
                }
                let content = fs_err::tokio::read_to_string(entry.path())
                    .await
                    .map_err(color_eyre::Report::new)?;
                Ok(Some((path, content)))
            } else {
                Ok(None)
            }
        })
        .map_ok(async |x| Ok(x))
        .try_buffer_unordered(8)
        .try_collect()
        .await?;
    file_contents
        .into_iter()
        .map(|(p, s)| downloads_from_file(input_dir, p, s))
        .flatten()
        .collect()
}

fn downloads_from_file(
    input_dir: &Utf8Path,
    file_path: Utf8PathBuf,
    file_content: String,
) -> Vec<Result<Download, color_eyre::Report>> {
    let file_parent = file_path.parent().unwrap_or(&file_path);
    let relative_path = pathdiff::diff_utf8_paths(file_parent, input_dir).unwrap();
    file_content
        .split_ascii_whitespace()
        .map(|s| {
            Download::try_from(s)
                .map(|mut d| {
                    d.filename = relative_path.join(d.filename).into_string();
                    d
                })
                .map_err(color_eyre::Report::new)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_urls () {
        let temp = TempDir::new().unwrap();
        fs_err::tokio::write(
            temp.path().join("a.txt"),
            "https://example.org/a.dat\nhttps://example.org/b.dat"
        ).await.unwrap();
        let subdir = temp.path().join("subdir");
        fs_err::tokio::create_dir(&subdir).await.unwrap();
        fs_err::tokio::write(
            subdir.join("x.txt"),
            "https://example.org/x.dat\nhttps://example.org/y.dat"
        ).await.unwrap();

        let downloads = read_urls(Utf8Path::from_path(temp.path()).unwrap()).await.unwrap();
        let actual: HashSet<_> = downloads.iter().map(|d| d.url.as_str()).collect();
        let expected: HashSet<_> = ["https://example.org/a.dat", "https://example.org/b.dat", "https://example.org/x.dat", "https://example.org/y.dat"].into_iter().collect();
        assert_eq!(actual, expected);
    }
}