use fltk::{app, prelude::*, window::Window, *};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use walkdir::WalkDir;
enum DisplayCommands {
    PrintPaths(Vec<PathBuf>),
    Clear,
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut window = Window::default()
        .with_size(600, 600)
        .center_screen()
        .with_label("Sweeper");
    window.make_resizable(true);

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
            remove_odd_files(files);
        }
        dialog::alert(0, 0, "All odd files are deleted!");
        clear_display.send(DisplayCommands::Clear).unwrap();
        clean_btn_d.deactivate();
    });

    // Here, we are updating TextDisplay, showing a user which files we will remove.
    while app.wait() {
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

fn remove_odd_files(files: Vec<PathBuf>) {
    for file in files {
        fs::remove_file(file).unwrap();
    }
}
