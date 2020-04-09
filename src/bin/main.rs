use collect_memories::{
    copy_files, retrieve_files_recursively, reverse_file_paths, FileSystemItem, ReversePath,
};
use cursive::align::*;
use cursive::event::*;
use cursive::theme::*;
use cursive::traits::*;
use cursive::views::*;
use cursive::Cursive;
use cursive_tree_view::{Placement, TreeView};
use nfd;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let mut siv = setup_main_ui();
    siv.run();
}

static DEFAULT_EXTENSIONS: &[&str; 10] = &[
    "jpeg", "jpg", "bmp", "gif", "png", "avi", "mp4", "mpg", "mpeg", "wmv",
];

fn setup_main_ui() -> Cursive {
    let mut s = Cursive::default();
    let mut layout = LinearLayout::vertical();
    layout.add_child(DummyView);
    layout.add_child(TextView::new("List of file extensions with memories:"));

    let mut extension_list = SelectView::<String>::new();
    extension_list.add_all_str(DEFAULT_EXTENSIONS.iter().map(|s| *s));

    let extension_list = extension_list
        .with_name("extension_list")
        .min_width(16)
        .min_height(8);

    let buttons = LinearLayout::vertical()
        .child(DummyView)
        .child(Button::new("Add", |s: &mut Cursive| add_extension(s)))
        .child(Button::new("Remove", |s: &mut Cursive| delete_name(s)));

    let extension_layout = LinearLayout::vertical().child(
        LinearLayout::horizontal()
            .child(Panel::new(extension_list))
            .child(buttons),
    );

    layout.add_child(extension_layout);
    layout.add_child(TextView::new("Path to directory:"));
    layout.add_child(
        LinearLayout::vertical()
            .child(
                EditView::new()
                    .content(".")
                    .with_name("input_dir_path")
                    .min_width(30),
            )
            .child(Button::new("Pick directory", |s: &mut Cursive| {
                pick_directory(s, "input_dir_path")
            })),
    );

    layout.add_child(DummyView);
    layout.add_child(
        TextView::new("Files are found by case insensitive extension matching.")
            .effect(Effect::Italic),
    );

    s.add_layer(
        Dialog::around(layout)
            .h_align(HAlign::Center)
            .title("Collect memories")
            .button("Scan for memories", scan_items_ui)
            .button("Quit", |s: &mut Cursive| s.quit()),
    );

    s
}

fn pick_directory(s: &mut Cursive, path_name: &str) {
    let result = nfd::open_pick_folder(None).unwrap();

    s.clear();
    s.refresh();
    use nfd::Response;
    match result {
        Response::Okay(file_path) => {
            let mut dir_path_view: ViewRef<EditView> = s.find_name(path_name).unwrap();
            dir_path_view.set_content(file_path);
        }
        _ => {}
    }
}

fn add_extension(s: &mut Cursive) {
    fn ok(s: &mut Cursive, extension_name: &str) {
        s.call_on_name("extension_list", |view: &mut SelectView| {
            view.add_item_str(extension_name.trim());
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("txt_extension")
                .fixed_width(10),
        )
        .title("Enter a new extension")
        .button("Ok", |s: &mut Cursive| {
            let extension = s
                .call_on_name("txt_extension", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, extension.as_ref())
        })
        .button("Cancel", |s: &mut Cursive| {
            s.pop_layer();
        }),
    );
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("extension_list").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove from extension list!")),
        Some(focus) => {
            select.remove_item(focus);
        }
    }
}

fn scan_items_ui(s: &mut Cursive) {
    let cb = s.cb_sink().clone();

    let input_path = PathBuf::from(
        s.find_name::<EditView>("input_dir_path")
            .unwrap()
            .get_content()
            .as_ref(),
    );
    let extensions: HashSet<String> = s
        .find_name::<SelectView<String>>("extension_list")
        .unwrap()
        .iter()
        .map(|(_, value)| value.trim().to_lowercase())
        .collect();

    s.pop_layer();

    let mut layout = LinearLayout::vertical();
    layout.add_child(TextView::new("Scanning..."));
    layout.add_child(TextView::new("").with_name("progress_file"));

    // And we start the worker thread.
    thread::spawn(move || {
        let last_update = std::cell::RefCell::new(Instant::now());

        let files = retrieve_files_recursively(
            &input_path,
            &|file: &PathBuf| -> bool {
                match file.extension() {
                    Some(extension) => {
                        extensions.contains(&extension.to_string_lossy().to_lowercase())
                    }
                    None => false,
                }
            },
            &|file_path: &PathBuf| {
                if last_update.borrow().elapsed() <= Duration::from_millis(1000 / 30) {
                    return;
                }
                last_update.replace(Instant::now());

                let file_path = file_path.to_string_lossy().into_owned();

                cb.send(Box::new(move |s: &mut Cursive| {
                    s.find_name::<TextView>("progress_file")
                        .unwrap()
                        .set_content(file_path)
                }))
                .unwrap();
            },
        );

        cb.send(Box::new(move |s: &mut Cursive| {
            list_files_found(s, files, input_path)
        }))
        .unwrap();
    });

    s.add_layer(Dialog::around(layout).title("Collect memories"));

    s.set_autorefresh(true);
}

fn list_files_found(
    s: &mut Cursive,
    files: io::Result<Option<FileSystemItem>>,
    input_path: PathBuf,
) {
    s.pop_layer();
    let mut layout = LinearLayout::vertical();
    layout.add_child(DummyView);

    match files {
        Ok(files) => match files {
            Some(files) => {
                layout.add_child(DummyView);
                layout.add_child(TextView::new("Memories found:"));
                tree_edit_part(s, &mut layout, &files, input_path);
            }
            None => {
                layout.add_child(TextView::new("No files with memories found!"));
            }
        },
        Err(err) => {
            layout.add_child(TextView::new("Failed to parse directories due to Error:"));
            layout.add_child(TextView::new(format!("{}", err)));
        }
    };

    s.add_layer(
        Dialog::around(layout)
            .h_align(HAlign::Center)
            .title("Collect memories")
            .button("Quit", |s: &mut Cursive| s.quit()),
    );
}

fn remove_active_subtree(s: &mut Cursive) {
    let mut tree_view = match s.find_name::<TreeView<TreeViewItem>>("tree_view") {
        Some(x) => x,
        None => return,
    };
    if let Some(row) = tree_view.row() {
        tree_view.remove_item(row);
    }
}

fn tree_edit_part(
    s: &mut Cursive,
    layout: &mut LinearLayout,
    files: &FileSystemItem,
    input_path: PathBuf,
) {
    s.add_global_callback(Event::Char('r'), remove_active_subtree);
    let tree_view = generate_tree_view(&files).with_name("tree_view");

    layout.add_child(
        LinearLayout::horizontal().child(tree_view).child(
            LinearLayout::vertical()
                .child(DummyView)
                .child(Button::new(
                    "Remove selected subtree from found file list!",
                    remove_active_subtree,
                ))
                .child(TextView::new("Shortcut: Press r"))
                .child(DummyView)
                .child(DummyView)
                .child(DummyView)
                .child(TextView::new("Path to output directory:"))
                .child(
                    LinearLayout::vertical()
                        .child(
                            EditView::new()
                                .content(".")
                                .with_name("output_dir_path")
                                .min_width(30),
                        )
                        .child(Button::new("Pick directory", |s: &mut Cursive| {
                            pick_directory(s, "output_dir_path")
                        })),
                )
                .child(DummyView)
                .child(Button::new("Copy memories", move |s: &mut Cursive| {
                    let output_path = PathBuf::from(
                        s.find_name::<EditView>("output_dir_path")
                            .unwrap()
                            .get_content()
                            .as_ref(),
                    );

                    let copied_input_path = input_path.clone();
                    s.add_layer(
                        Dialog::around(
                            LinearLayout::vertical()
                                .child(TextView::new(
                                    "Do you really want to copy found files to the following path?",
                                ))
                                .child(TextView::new(format!(
                                    "To: {}",
                                    output_path.to_string_lossy().into_owned()
                                ))),
                        )
                        .button("Ok", move |s: &mut Cursive| {
                            s.pop_layer();
                            let mut tree_view =
                                s.find_name::<TreeView<TreeViewItem>>("tree_view").unwrap();
                            let items = tree_view.take_items();
                            copy_items_ui(
                                s,
                                items
                                    .into_iter()
                                    .filter(|item| !item.directory)
                                    .map(|item| item.path)
                                    .collect(),
                                copied_input_path.clone(),
                                output_path.clone(),
                            );
                        })
                        .dismiss_button("Cancel")
                        .title("Collect memories"),
                    );
                })),
        ),
    );
}

fn insert_tree_view(
    reverse_path: &ReversePath,
    location: &FileSystemItem,
    tree: &mut TreeView<TreeViewItem>,
    placement_type: Placement,
    parent_row: usize,
) {
    match location {
        FileSystemItem::File => {
            tree.insert_item(
                TreeViewItem {
                    directory: false,
                    path: reverse_path.clone(),
                },
                placement_type,
                parent_row,
            );
        }
        FileSystemItem::Directory(root_dir) => {
            let row = match tree.insert_item(
                TreeViewItem {
                    path: reverse_path.clone(),
                    directory: true,
                },
                placement_type,
                parent_row,
            ) {
                Some(out) => out,
                None => panic!("{:?}", tree),
            };
            for (name, dir) in root_dir.content() {
                insert_tree_view(
                    &ReversePath::new_from_prefix(reverse_path, name),
                    dir,
                    tree,
                    Placement::LastChild,
                    row,
                );
            }
        }
    };
}

fn generate_tree_view(files: &FileSystemItem) -> TreeView<TreeViewItem> {
    let mut tree = TreeView::new();
    if let FileSystemItem::Directory(root_dir) = files {
        for (name, dir) in root_dir.content() {
            insert_tree_view(&ReversePath::new(name), dir, &mut tree, Placement::After, 0);
        }
    }
    tree
}

#[derive(Debug)]
struct TreeViewItem {
    directory: bool,
    path: ReversePath,
}

impl std::fmt::Display for TreeViewItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path.last_member().to_string_lossy().as_ref())
    }
}

fn copy_items_ui(
    s: &mut Cursive,
    paths: Vec<ReversePath>,
    input_path: PathBuf,
    output_path: PathBuf,
) {
    let cb = s.cb_sink().clone();
    s.pop_layer();

    let mut layout = LinearLayout::vertical();
    layout.add_child(TextView::new("Copying..."));
    layout.add_child(TextView::new(format!(
        "From: {}",
        &input_path.to_string_lossy()
    )));
    layout.add_child(TextView::new(format!(
        "To: {}",
        &output_path.to_string_lossy()
    )));
    layout.add_child(TextView::new("").with_name("copy_progress_file"));

    let file_tree = reverse_file_paths(&paths);

    // And we start the worker thread.
    thread::spawn(move || {
        let last_update = std::cell::RefCell::new(Instant::now());

        let files_copied = copy_files(
            &input_path,
            &output_path,
            &FileSystemItem::Directory(file_tree),
            &PathBuf::new(),
            &|file_path: &PathBuf| {
                if last_update.borrow().elapsed() <= Duration::from_millis(1000 / 30) {
                    return;
                }
                last_update.replace(Instant::now());

                let file_path = file_path.to_string_lossy().into_owned();
                cb.send(Box::new(move |s: &mut Cursive| {
                    s.find_name::<TextView>("copy_progress_file")
                        .unwrap()
                        .set_content(file_path)
                }))
                .unwrap();
            },
        );
        {}

        cb.send(Box::new(move |s: &mut Cursive| {
            done_ui(s, files_copied, input_path, output_path)
        }))
        .unwrap();
    });

    s.add_layer(Dialog::around(layout).title("Collect memories"));

    s.set_autorefresh(true);
}

fn done_ui(s: &mut Cursive, result: io::Result<u32>, input_path: PathBuf, output_path: PathBuf) {
    s.pop_layer();
    let message = match result {
        Ok(cnt) => format!("{} memories copied!", cnt),
        Err(err) => format!("Operation failed\n{}", err),
    };

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(message))
                .child(TextView::new(format!(
                    "From: {}",
                    input_path.to_string_lossy()
                )))
                .child(TextView::new(format!(
                    "To: {}",
                    output_path.to_string_lossy()
                ))),
        )
        .title("Collect memories")
        .button("Close", |s| s.quit()),
    );
}
