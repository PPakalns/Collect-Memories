use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default)]
pub struct Directory {
    content: HashMap<OsString, FileSystemItem>,
}

impl Directory {
    pub fn content(&self) -> &HashMap<OsString, FileSystemItem> {
        &self.content
    }
}

pub enum FileSystemItem {
    File,
    Directory(Directory),
}

pub fn retrieve_files_recursively<F1, F2>(
    path: &PathBuf,
    check: &F1,
    callback: &F2,
) -> io::Result<Option<FileSystemItem>>
where
    F1: Fn(&PathBuf) -> bool,
    F2: Fn(&PathBuf),
{
    let mut dir: Directory = Default::default();

    let read_dir_iter = match path.read_dir() {
        Ok(it) => it,
        Err(err) => {
            if err.kind() == io::ErrorKind::PermissionDenied {
                return Ok(None);
            } else {
                return Err(err);
            }
        }
    };

    for child in read_dir_iter {
        let child: fs::DirEntry = child?;
        let file_type = child.file_type()?;

        let item: FileSystemItem = if file_type.is_dir() {
            match retrieve_files_recursively(&child.path(), check, callback)? {
                Some(item) => item,
                None => continue,
            }
        } else if file_type.is_file() {
            let file_path = child.path();
            callback(&file_path);
            if check(&file_path) == false {
                continue;
            }
            FileSystemItem::File
        } else {
            continue;
        };

        use std::collections::hash_map::Entry;
        match dir.content.entry(
            child
                .path()
                .file_name()
                .expect("Non empty filename!")
                .to_owned(),
        ) {
            Entry::Occupied(_occupied) => {
                panic!(
                    "Multiple files with the same path '{}' in directory!",
                    child.path().to_string_lossy()
                );
            }
            Entry::Vacant(vacant) => {
                vacant.insert(item);
            }
        }
    }

    if dir.content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(FileSystemItem::Directory(dir)))
    }
}

#[derive(Debug)]
struct ReversePathPart {
    part: OsString,
    prefix: Option<Rc<ReversePathPart>>,
}

impl ReversePathPart {
    fn path(&self) -> PathBuf {
        match &self.prefix {
            Some(prefix) => {
                let mut path = prefix.path();
                path.push(&self.part);
                path
            }
            None => PathBuf::from(&self.part),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReversePath {
    last_part: Rc<ReversePathPart>,
}

impl ReversePath {
    pub fn new(part: &OsStr) -> ReversePath {
        ReversePath {
            last_part: Rc::new(ReversePathPart {
                part: part.to_owned(),
                prefix: None,
            }),
        }
    }

    pub fn new_from_prefix(prefix: &ReversePath, part: &OsStr) -> ReversePath {
        ReversePath {
            last_part: Rc::new(ReversePathPart {
                part: part.to_owned(),
                prefix: Some(prefix.last_part.clone()),
            }),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.last_part.path()
    }

    pub fn last_member(&self) -> &OsStr {
        &self.last_part.part
    }
}

fn build_directory_tree<'a>(
    root_dir: &'a mut Directory,
    part: &ReversePathPart,
) -> &'a mut Directory {
    let last_part = match part.prefix.as_ref() {
        Some(part) => build_directory_tree(root_dir, part.as_ref()),
        None => root_dir,
    };

    let directory =
        match last_part
            .content
            .entry(part.part.to_owned())
            .or_insert(FileSystemItem::Directory(Directory {
                ..Default::default()
            })) {
            FileSystemItem::Directory(x) => x,
            _ => panic!("Unexpected file and directory name collision!"),
        };

    directory
}

fn build_file_tree(root_dir: &mut Directory, part: &ReversePathPart) {
    let last_part = match part.prefix.as_ref() {
        Some(part) => build_directory_tree(root_dir, part.as_ref()),
        None => root_dir,
    };
    last_part
        .content
        .insert(part.part.to_owned(), FileSystemItem::File);
}

pub fn reverse_file_paths(paths: &Vec<ReversePath>) -> Directory {
    let mut dir = Default::default();
    for path in paths {
        build_file_tree(&mut dir, path.last_part.as_ref());
    }
    dir
}

pub fn copy_files<F>(
    input_path: &PathBuf,
    output_path: &PathBuf,
    item: &FileSystemItem,
    current_path: &PathBuf,
    callback: &F,
) -> io::Result<u32>
where
    F: Fn(&PathBuf),
{
    let destination_path = output_path.join(current_path);

    match item {
        FileSystemItem::File => {
            if destination_path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "Destination file {} already exists!",
                        destination_path.to_string_lossy()
                    ),
                ));
            }
            callback(&destination_path);
            fs::copy(input_path.join(current_path), destination_path)?;
            Ok(1)
        }
        FileSystemItem::Directory(directory) => {
            if !destination_path.exists() {
                fs::create_dir(destination_path)?;
            }
            let mut file_cnt = 0;
            for (child_path, child_item) in directory.content.iter() {
                let child_path = current_path.join(child_path);
                file_cnt += copy_files(input_path, output_path, child_item, &child_path, callback)?;
            }
            Ok(file_cnt)
        }
    }
}
