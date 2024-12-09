use std::{
  future::Future,
  path::{Path, PathBuf},
  pin::Pin,
};

#[tokio::main]
async fn main() {
  let path = std::env::args().nth(1).unwrap_or(".".to_string());
  let path = path.as_str();
  let path = Path::new(path);

  if !path.exists() {
    eprintln!("Path does not exist");
    return;
  }

  if path.is_file() {
    move_file(path.to_path_buf(), path.parent().unwrap().to_path_buf()).await;
  } else {
    run_on_dir(path).await;
  }
}

async fn run_on_dir(path: &Path) {
  walk_dir(path, &|file_path| {
    Box::pin(async {
      let path = path.to_path_buf();
      move_file(file_path, path).await;
    })
  })
  .await;
}

async fn move_file(path: PathBuf, working_dir: PathBuf) {
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

  let directory = working_dir.join(year).join(month);

  if !directory.exists() {
    tokio::fs::create_dir_all(&directory).await.expect("Error creating directory");
  }

  let new_path = directory.join(filename.to_string());

  tokio::fs::rename(&path, &new_path).await.expect("Error moving file");
}

async fn walk_dir<'a, C>(path: &Path, cb: &C)
where
  C: Fn(PathBuf) -> Pin<Box<dyn Future<Output = ()> + 'a>>,
{
  Box::pin(async move {
    if path.is_dir() {
      let mut entries = tokio::fs::read_dir(path).await.unwrap();

      while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();

        if path.is_dir() {
          walk_dir(&path, cb).await;
        } else {
          cb(path).await;
        }
      }
    }
  })
  .await
}
