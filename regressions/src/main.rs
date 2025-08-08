use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::cmp::min;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
pub struct FilesList {
    checksum: String,
    file_name: String,
}

impl FilesList {
    pub fn new(checksum: String, file_name: String) -> FilesList {
        FilesList {
            checksum,
            file_name,
        }
    }

    pub fn download_url(&self) -> String {
        format!(
            "https://raw.pixls.us/download/data/{}",
            self.file_name.replace(' ', "%20")
        )
    }

    pub fn save_path(&self) -> PathBuf {
        PathBuf::from(&format!("./files/{}", self.checksum))
    }
}

#[derive(Debug)]
struct DownloadCallbackProgress {
    /// Total bytes downloaded so far.
    bytes_downloaded: u64,
    /// Total size of the file in bytes.
    total_bytes: u64,
}

#[tokio::main]
async fn main() {
    // download_task().await;
    // generate_known_outputs();
    verify_outputs();
}

fn verify_outputs() {
    let dir = Path::new("./files").read_dir().unwrap();
    let count = Arc::new(AtomicUsize::new(0));

    dir.par_bridge().for_each(|file| {
        let file = file.unwrap();

        let mut known = File::open(format!(
            "./known_good_outputs/{}",
            file.file_name().to_str().unwrap()
        ))
        .unwrap();

        let mut known_data = vec![];

        known.read_to_end(&mut known_data).unwrap();

        if known_data.is_empty() {
            return;
        }

        let new_output = {
            if let Ok(raw) = rawloader::decode_file(file.path()) {
                let mut lines = vec![];

                lines.push(format!("make = {}", raw.make));
                lines.push(format!("model = {}", raw.model));
                lines.push(format!("width = {}", raw.width));
                lines.push(format!("height = {}", raw.height));
                lines.push(format!("cpp = {}", raw.cpp));
                lines.push(format!("wb_coeffs = {:?}", raw.wb_coeffs));
                lines.push(format!("whitelevels = {:?}", raw.whitelevels));
                lines.push(format!("blacklevels = {:?}", raw.blacklevels));
                lines.push(format!("xyz_to_cam = {:?}", raw.xyz_to_cam));
                lines.push(format!("cfa = {}", raw.cfa));
                lines.push(format!("crops = {:?}", raw.crops));
                lines.push(format!("blackareas = {:?}", raw.blackareas));
                lines.push(format!("orientation = {:?}", raw.orientation));

                lines.join("\n").into_bytes()
            } else {
                vec![]
            }
        };

        if known_data != new_output {
            count.fetch_add(1, Relaxed);
            eprintln!("{} doesn't match", file.file_name().to_str().unwrap());

            let diff = TextDiff::from_lines(
                from_utf8(&known_data).unwrap(),
                from_utf8(&new_output).unwrap(),
            );

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => {
                        continue;
                    }
                };
                print!("{}{}", sign, change);
            }
        }
    });

    println!("{} mismatches", count.load(Relaxed));
}

fn generate_known_outputs() {
    let dir = Path::new("./files").read_dir().unwrap();

    dir.par_bridge().for_each(|file| {
        let file = file.unwrap();

        let mut new_file = File::create(format!(
            "./known_good_outputs/{}",
            file.file_name().to_str().unwrap()
        ))
        .unwrap();

        if let Ok(raw) = rawloader::decode_file(file.path()) {
            let mut lines = vec![];

            lines.push(format!("make = {}", raw.make));
            lines.push(format!("model = {}", raw.model));
            lines.push(format!("width = {}", raw.width));
            lines.push(format!("height = {}", raw.height));
            lines.push(format!("cpp = {}", raw.cpp));
            lines.push(format!("wb_coeffs = {:?}", raw.wb_coeffs));
            lines.push(format!("whitelevels = {:?}", raw.whitelevels));
            lines.push(format!("blacklevels = {:?}", raw.blacklevels));
            lines.push(format!("xyz_to_cam = {:?}", raw.xyz_to_cam));
            lines.push(format!("cfa = {}", raw.cfa));
            lines.push(format!("crops = {:?}", raw.crops));
            lines.push(format!("blackareas = {:?}", raw.blackareas));
            lines.push(format!("orientation = {:?}", raw.orientation));

            new_file.write_all(lines.join("\n").as_bytes()).unwrap();
        }
    });
}

async fn download_task() {
    let file = File::open("./filelist.sha1").unwrap();
    let reader = BufReader::new(file);

    let mut reader = reader.lines();

    let mut files = vec![];

    while let Some(Ok(line)) = reader.next() {
        if let [hash, rest] = line.split('*').collect::<Vec<&str>>()[..] {
            files.push(FilesList::new(hash.trim().to_string(), rest.to_string()));
        }
    }

    download_batch(files).await.unwrap()
}

async fn download_batch(urls: Vec<FilesList>) -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = async_channel::bounded(25);

    let mut task_set = JoinSet::new();

    for url in urls {
        task_set.spawn({
            let sender = sender.clone();

            async move {
                sender.send(url).await.unwrap();
            }
        });
    }

    let progress = MultiProgress::new();

    {
        for _ in 0..25 {
            let receiver = receiver.clone();
            task_set.spawn({
                let progress = progress.clone();
                let receiver = receiver.clone();

                async move {
                    while let Ok(files) = receiver.recv().await {
                        let progress = progress.clone();

                        download_file(files.download_url(), files.save_path(), progress)
                            .await
                            .unwrap();
                    }
                }
            });
        }
    }

    task_set.join_all().await;

    Ok(())
}

async fn download_file(
    url: impl AsRef<str> + Display,
    path: impl AsRef<Path> + Debug,
    progress: MultiProgress,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut response = client.get(url.as_ref()).send().await?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));

    progress.add(pb.clone());

    let mut file = File::create(&path).or(Err(format!("Failed to create file '{path:?}'")))?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish();

    progress.remove(&pb);

    Ok(())
}
