use fltk::enums::{FrameType, Shortcut};
use fltk::{app, prelude::*, window::Window, *};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::{fs, io};
use walkdir::WalkDir;

const ABOUT: &str = include_str!("about.html");

enum DisplayCommands {
    PrintPaths(Vec<PathBuf>),
    Clear,
}

#[derive(Clone)]
enum MenuChoice {
    About,
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut window = Window::default()
        .with_size(600, 600)
        .center_screen()
        .with_label("Sweeper");
    window.make_resizable(true);

    let mut menu = menu::SysMenuBar::default().with_size(800, 35);
    menu.set_frame(FrameType::FlatBox);

    let (menu_sender, menu_receiver) = app::channel();
    menu.add_emit(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        menu_sender,
        MenuChoice::About,
    );

    let mut buf = text::TextBuffer::default();
    let mut text_display = text::TextDisplay::default()
        .with_size(400, 400)
        .center_of_parent()
        .with_label("Files to remove");
    text_display.set_buffer(buf.clone());

    let mut scan_btn = button::Button::new(140, 510, 150, 30, "Scan directory(s)");
    let mut clean_btn = button::Button::new(320, 510, 150, 30, "Clean Up!");
    clean_btn.deactivate();

    window.end();
    window.show();

    let (display_send, display_recv) = channel();
    let (file_list_sender, file_list_receiver) = channel();

    let clear_display = display_send.clone();

    // The button that initiates the scanning process is used to locate all odd files.
    // Odd files are those without a pair; for example, there is a foo.jpeg file but no foo.nef file.
    scan_btn.set_callback(move |_| {
        let mut dialog =
            dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseMultiDir);
        dialog.show();

        if dialog.filenames().is_empty() {
            dialog::alert(0, 0, "You have to choose a directory!");
            return;
        }

        display_send.send(DisplayCommands::Clear).unwrap();
        let paths = find_odd_files(&dialog.filenames());
        match paths {
            None => {
                dialog::alert(
                    0,
                    0,
                    "This directory does not contain any odd files, or jpg, jpeg, nef files in general.",
                );
            }
            Some(files_to_delete) => {
                display_send
                    .send(DisplayCommands::PrintPaths(files_to_delete.clone()))
                    .unwrap();
                file_list_sender.send(files_to_delete).unwrap();
            }
        }
    });

    // Clean up button
    let mut clean_btn_d = clean_btn.clone();
    clean_btn.set_callback(move |_| {
        if let Ok(files) = file_list_receiver.try_recv() {
            // Ask for confirmation only if not already confirmed
            if dialog::choice2(
                0,
                0,
                "You are going to delete the listed files! Be careful!",
                "Quit!",
                "Ok",
                "",
            ) == Some(0)
            {
                app::quit();
                return;
            }
            match remove_files(files) {
                Ok(_) => {}
                Err(_) => {
                    dialog::alert(0, 0, "Cannot remove files!");
                }
            };
        }
        dialog::alert(0, 0, "All odd files are deleted!");
        clear_display.send(DisplayCommands::Clear).unwrap();
        clean_btn_d.deactivate();
    });

    while app.wait() {
        // Handle menu selection
        if let Some(msg) = menu_receiver.recv() {
            match msg {
                MenuChoice::About => {
                    let mut help = dialog::HelpDialog::new(100, 100, 800, 800);
                    help.set_value(ABOUT);
                    help.show();
                    while help.shown() {
                        app::wait();
                    }
                }
            }
        }

        // Here, we are updating TextDisplay, showing a user which files we will remove.
        if let Ok(msg) = display_recv.try_recv() {
            match msg {
                DisplayCommands::PrintPaths(m) => {
                    for p in m {
                        buf.append(&p.to_string_lossy());
                        buf.append("\n");
                        clean_btn.activate();
                    }
                }
                DisplayCommands::Clear => {
                    buf.set_text("");
                }
            }
        }
    }
}

fn find_odd_files(dirs: &Vec<PathBuf>) -> Option<Vec<PathBuf>> {
    if dirs.is_empty() {
        return None;
    }

    let mut result: Vec<PathBuf> = vec![];
    for dir in dirs {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let f_name = entry.file_name().to_string_lossy();
            if f_name.ends_with(".jpeg") || f_name.ends_with(".jpg") {
                let mut n = PathBuf::from(entry.path());
                n.set_extension("nef");
                if !n.exists() {
                    result.push(PathBuf::from(entry.path()));
                }
            } else if f_name.ends_with("nef") {
                let mut jpeg = PathBuf::from(entry.path());
                jpeg.set_extension("jpeg");

                let mut jpg = PathBuf::from(entry.path());
                jpg.set_extension("jpg");

                if !jpeg.exists() && !jpg.exists() {
                    result.push(PathBuf::from(entry.path()));
                }
            }
        }
    }

    if result.is_empty() {
        return None;
    }

    Some(result)
}

fn remove_files(files: Vec<PathBuf>) -> io::Result<()> {
    for file in files {
        fs::remove_file(&file)?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{find_odd_files, remove_files};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn find_odd_files_test() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let dir_path = tmp_dir.path();
        let mut odd_files = vec![];

        // Generate files
        for i in 0..10 {
            let mut file = dir_path.join(PathBuf::from(format!("file_{i}.jpg")));
            let file_path = file.as_path();
            fs::File::create(&file_path).unwrap();
            if i % 2 == 0 {
                file.set_extension("nef");
                let file_path = file.as_path();
                fs::File::create(file_path).unwrap();
            } else {
                odd_files.push(file_path.to_owned());
            }
        }

        let dirs = vec![PathBuf::from(dir_path)];
        let mut res = find_odd_files(&dirs).unwrap();

        assert_eq!(res.len(), odd_files.len());

        res.sort();
        odd_files.sort();
        assert_eq!(res, odd_files);
    }

    #[test]
    fn find_odd_files_in_nested_directories_test() {
        let mut odd_files = vec![];

        // Generate root files
        let root_dir = tempfile::TempDir::new().unwrap();
        let root_dir_path = root_dir.path();
        for i in 0..10 {
            let mut file = root_dir_path.join(PathBuf::from(format!("file_{i}.jpg")));
            let file_path = file.as_path();
            fs::File::create(&file_path).unwrap();
            if i % 2 == 0 {
                file.set_extension("nef");
                let file_path = file.as_path();
                fs::File::create(file_path).unwrap();
            } else {
                odd_files.push(file_path.to_owned());
            }
        }

        // Generate nested files
        let nested_dir = tempfile::TempDir::new_in(root_dir_path).unwrap();
        let nested_dir_path = nested_dir.path();
        for i in 0..10 {
            let mut file = nested_dir_path.join(PathBuf::from(format!("file_{i}.jpg")));
            let file_path = file.as_path();
            fs::File::create(&file_path).unwrap();
            if i % 2 == 0 {
                odd_files.push(file_path.to_owned());
            } else {
                file.set_extension("nef");
                let file_path = file.as_path();
                fs::File::create(file_path).unwrap();
            }
        }

        let dirs = vec![PathBuf::from(root_dir_path)];
        let mut res = find_odd_files(&dirs).unwrap();

        assert_eq!(res.len(), odd_files.len());

        res.sort();
        odd_files.sort();
        assert_eq!(res, odd_files);
    }

    #[test]
    fn remove_odd_files_test() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let dir_path = tmp_dir.path();

        let mut files = vec![];
        let ext = ["jpg", "nef", "txt"];
        for e in ext {
            let file = dir_path.join(PathBuf::from(format!("file.{e}")));
            let file_path = file.as_path();
            fs::File::create(file_path).unwrap();
            files.push(PathBuf::from(file_path));
        }

        remove_files(files).unwrap();
        let is_empty = fs::read_dir(dir_path).unwrap().next().is_none();
        assert!(is_empty);
    }
}
