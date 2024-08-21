use std::{
  fs,
  path::{Path, PathBuf},
  sync::mpsc,
};

mod thread_pool;
use thread_pool::*;

fn main() {
  let (tx, rx) = mpsc::channel();

  walk_dir(Path::new("."), tx);

  let thread_pool = ThreadPool::default();

  while let Ok(path) = rx.recv() {
    thread_pool.execute(move || {
      move_file(path);
    });
  }
}

fn move_file(path: PathBuf) {
  let filename = path.file_name().unwrap().to_string_lossy();

  println!("Moving file: {}", &filename);

  let metadata = path.metadata().expect("Error getting file metadata");
  let timestamp = metadata
    .modified()
    .expect("Error getting file modification time")
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

  let date = chrono::DateTime::from_timestamp(timestamp as i64, 0);

  let date = match date {
    Some(date) => date,
    None => {
      eprintln!("Error parsing date for file: {}", &filename);
      return;
    }
  };

  let year = date.format("%Y").to_string();
  let month = date.format("%m").to_string();

  let directory = Path::new(".").join(year).join(month);

  if !directory.exists() {
    fs::create_dir_all(&directory).expect("Error creating directory");
  }

  let new_path = directory.join(filename.to_string());

  fs::rename(&path, &new_path).expect("Error moving file");
}

fn walk_dir(path: &Path, tx: mpsc::Sender<PathBuf>) {
  if path.is_dir() {
    for entry in fs::read_dir(path).unwrap() {
      let entry = entry.unwrap();
      let path = entry.path();

      walk_dir(&path, tx.clone());
    }
  } else if path.is_file() {
    tx.send(path.to_path_buf()).unwrap();
  }
}
