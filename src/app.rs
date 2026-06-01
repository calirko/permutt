use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GBox, Button, DropTarget, FileDialog,
    FileFilter, Label, ListBox, ListBoxRow, Orientation, Overlay, Revealer, RevealerTransitionType,
    ScrolledWindow, SelectionMode, Separator, Spinner, Stack,
};

use crate::converter::{self, ConvertJob};
use crate::output_picker::OutputPicker;

const APP_ID: &str = "com.calirko.permutt";

pub fn build_app() -> Application {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Permutt")
        .default_width(900)
        .default_height(600)
        .build();

    let state = Arc::new(Mutex::new(AppState::default()));

    let root = GBox::new(Orientation::Horizontal, 0);

    // ── Left panel ────────────────────────────────────────────────────────────
    let left = build_left_panel(state.clone());
    left.set_hexpand(true);
    left.set_width_request(420);

    let sep = Separator::new(Orientation::Vertical);

    // ── Right panel ───────────────────────────────────────────────────────────
    let right = build_right_panel(state.clone());
    right.set_width_request(320);

    root.append(&left);
    root.append(&sep);
    root.append(&right);

    window.set_child(Some(&root));
    window.present();
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct AppState {
    files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
}

// ── Left panel (file input) ────────────────────────────────────────────────────

fn build_left_panel(state: Arc<Mutex<AppState>>) -> GBox {
    let panel = GBox::new(Orientation::Vertical, 0);
    panel.add_css_class("left-panel");

    // Header bar
    let header = GBox::new(Orientation::Horizontal, 8);
    header.add_css_class("panel-header");
    header.set_margin_start(16);
    header.set_margin_end(16);
    header.set_margin_top(12);
    header.set_margin_bottom(12);

    let title = Label::new(Some("Input Files"));
    title.add_css_class("panel-title");
    title.set_halign(Align::Start);
    title.set_hexpand(true);

    let browse_btn = Button::with_label("Browse…");
    browse_btn.add_css_class("pill-button");

    header.append(&title);
    header.append(&browse_btn);

    // Drop zone / file list (stacked)
    let stack = Stack::new();

    let drop_hint = build_drop_hint();
    stack.add_named(&drop_hint, Some("hint"));

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::Multiple);
    list.add_css_class("file-list");
    scroll.set_child(Some(&list));
    stack.add_named(&scroll, Some("list"));

    stack.set_visible_child_name("hint");

    // Wire drag-and-drop
    let drop_target = DropTarget::new(gtk::gio::File::static_type(), gtk::gdk::DragAction::COPY);
    drop_target.set_types(&[gtk::gdk::FileList::static_type()]);

    let stack_clone = stack.clone();
    let list_clone = list.clone();
    let state_drop = state.clone();

    drop_target.connect_drop(move |_, value, _, _| {
        if let Ok(file_list) = value.get::<gtk::gdk::FileList>() {
            let paths: Vec<PathBuf> = file_list
                .files()
                .iter()
                .filter_map(|f| f.path())
                .filter(|p| converter::is_supported_input(p))
                .collect();

            if paths.is_empty() {
                return false;
            }

            add_files_to_list(&list_clone, &paths);

            if let Ok(mut s) = state_drop.lock() {
                s.files.extend(paths);
            }

            stack_clone.set_visible_child_name("list");
            true
        } else {
            false
        }
    });

    let overlay = Overlay::new();
    overlay.set_child(Some(&stack));
    overlay.add_controller(drop_target);

    // Drop border visual
    let drop_border = GBox::new(Orientation::Vertical, 0);
    drop_border.add_css_class("drop-overlay");
    drop_border.set_can_target(false);
    overlay.add_overlay(&drop_border);

    // Wire browse button
    let list_browse = list.clone();
    let stack_browse = stack.clone();
    let state_browse = state.clone();

    browse_btn.connect_clicked(move |btn| {
        let filter = FileFilter::new();
        filter.add_mime_type("image/*");
        filter.set_name(Some("Images"));

        let dialog = FileDialog::builder()
            .title("Select Images")
            .default_filter(&filter)
            .build();

        let list_inner = list_browse.clone();
        let stack_inner = stack_browse.clone();
        let state_inner = state_browse.clone();
        let win = btn
            .root()
            .and_then(|r| r.downcast::<ApplicationWindow>().ok());

        dialog.open_multiple(
            win.as_ref(),
            gtk::gio::Cancellable::NONE,
            move |result| {
                if let Ok(file_list) = result {
                    let paths: Vec<PathBuf> = (0..file_list.n_items())
                        .filter_map(|i| {
                            file_list
                                .item(i)
                                .and_then(|o| o.downcast::<gtk::gio::File>().ok())
                                .and_then(|f| f.path())
                        })
                        .filter(|p| converter::is_supported_input(p))
                        .collect();

                    if paths.is_empty() {
                        return;
                    }

                    add_files_to_list(&list_inner, &paths);

                    if let Ok(mut s) = state_inner.lock() {
                        s.files.extend(paths);
                    }

                    stack_inner.set_visible_child_name("list");
                }
            },
        );
    });

    // Clear button at bottom
    let clear_btn = Button::with_label("Clear all");
    clear_btn.add_css_class("flat");
    clear_btn.set_margin_start(16);
    clear_btn.set_margin_end(16);
    clear_btn.set_margin_top(8);
    clear_btn.set_margin_bottom(8);

    let list_clear = list.clone();
    let stack_clear = stack.clone();
    let state_clear = state.clone();
    clear_btn.connect_clicked(move |_| {
        while let Some(row) = list_clear.first_child() {
            list_clear.remove(&row);
        }
        if let Ok(mut s) = state_clear.lock() {
            s.files.clear();
        }
        stack_clear.set_visible_child_name("hint");
    });

    panel.append(&header);
    panel.append(&overlay);
    panel.append(&clear_btn);
    panel
}

fn build_drop_hint() -> GBox {
    let b = GBox::new(Orientation::Vertical, 12);
    b.set_valign(Align::Center);
    b.set_halign(Align::Center);
    b.set_vexpand(true);
    b.add_css_class("drop-hint");

    let icon = Label::new(Some("⬆"));
    icon.add_css_class("drop-icon");

    let primary = Label::new(Some("Drop images here"));
    primary.add_css_class("drop-primary");

    let secondary = Label::new(Some("or click Browse to select files"));
    secondary.add_css_class("drop-secondary");

    b.append(&icon);
    b.append(&primary);
    b.append(&secondary);
    b
}

fn add_files_to_list(list: &ListBox, paths: &[PathBuf]) {
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        let row_box = GBox::new(Orientation::Horizontal, 8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);

        let label = Label::new(Some(&name));
        label.set_halign(Align::Start);
        label.set_hexpand(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_uppercase();
        let badge = Label::new(Some(&ext));
        badge.add_css_class("format-badge");

        row_box.append(&label);
        row_box.append(&badge);

        let row = ListBoxRow::new();
        row.set_child(Some(&row_box));
        list.append(&row);
    }
}

// ── Right panel (output settings + convert) ────────────────────────────────────

fn build_right_panel(state: Arc<Mutex<AppState>>) -> GBox {
    let panel = GBox::new(Orientation::Vertical, 0);
    panel.add_css_class("right-panel");

    let header = Label::new(Some("Output"));
    header.add_css_class("panel-title");
    header.set_halign(Align::Start);
    header.set_margin_start(16);
    header.set_margin_top(12);
    header.set_margin_bottom(12);

    panel.append(&header);
    panel.append(&Separator::new(Orientation::Horizontal));

    // Format picker
    let picker = OutputPicker::new();
    panel.append(picker.widget());

    panel.append(&Separator::new(Orientation::Horizontal));

    // Output directory row
    let dir_row = build_dir_row(state.clone());
    panel.append(&dir_row);

    panel.append(&Separator::new(Orientation::Horizontal));

    // Spacer
    let spacer = GBox::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    panel.append(&spacer);

    // Status revealer
    let status_label = Label::new(None);
    status_label.add_css_class("status-label");
    status_label.set_margin_start(16);
    status_label.set_margin_end(16);
    status_label.set_wrap(true);

    let status_rev = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .child(&status_label)
        .build();
    panel.append(&status_rev);

    // Convert button
    let convert_row = GBox::new(Orientation::Horizontal, 8);
    convert_row.set_margin_start(16);
    convert_row.set_margin_end(16);
    convert_row.set_margin_top(12);
    convert_row.set_margin_bottom(16);

    let spinner = Spinner::new();
    let convert_btn = Button::with_label("Convert");
    convert_btn.add_css_class("suggested-action");
    convert_btn.add_css_class("pill-button");
    convert_btn.set_hexpand(true);

    convert_row.append(&spinner);
    convert_row.append(&convert_btn);
    panel.append(&convert_row);

    // Wire convert button
    let picker_ref = picker.clone();
    let state_convert = state.clone();
    let spinner_ref = spinner.clone();
    let btn_ref = convert_btn.clone();
    let status_label_ref = status_label.clone();
    let status_rev_ref = status_rev.clone();

    convert_btn.connect_clicked(move |_| {
        let (files, output_dir, fmt) = {
            let s = state_convert.lock().unwrap();
            (
                s.files.clone(),
                s.output_dir.clone(),
                picker_ref.selected_format(),
            )
        };

        if files.is_empty() {
            show_status(&status_label_ref, &status_rev_ref, "No input files selected.", false);
            return;
        }

        let fmt = match fmt {
            Some(f) => f,
            None => {
                show_status(
                    &status_label_ref,
                    &status_rev_ref,
                    "Pick an output format first.",
                    false,
                );
                return;
            }
        };

        let out_dir = output_dir.unwrap_or_else(default_output_dir);

        spinner_ref.start();
        btn_ref.set_sensitive(false);

        let (tx, rx) = async_channel::bounded(1);

        let jobs: Vec<ConvertJob> = files
            .into_iter()
            .map(|input| ConvertJob {
                input,
                output_dir: out_dir.clone(),
                format: fmt,
            })
            .collect();

        std::thread::spawn(move || {
            let results: Vec<_> = jobs.iter().map(converter::convert).collect();
            let _ = tx.send_blocking(results);
        });

        let spinner_done = spinner_ref.clone();
        let btn_done = btn_ref.clone();
        let status_label_done = status_label_ref.clone();
        let status_rev_done = status_rev_ref.clone();

        glib::MainContext::default().spawn_local(async move {
            if let Ok(results) = rx.recv().await {
                spinner_done.stop();
                btn_done.set_sensitive(true);

                let ok = results.iter().filter(|r| r.success).count();
                let failures: Vec<&str> = results
                    .iter()
                    .filter_map(|r| r.error.as_deref())
                    .collect();

                let msg = if failures.is_empty() {
                    format!("Done! {} file(s) converted.", ok)
                } else {
                    format!(
                        "{} converted, {} failed: {}",
                        ok,
                        failures.len(),
                        failures[0]
                    )
                };

                show_status(&status_label_done, &status_rev_done, &msg, failures.is_empty());
            }
        });
    });

    panel
}

fn build_dir_row(state: Arc<Mutex<AppState>>) -> GBox {
    let row = GBox::new(Orientation::Vertical, 4);
    row.set_margin_start(16);
    row.set_margin_end(16);
    row.set_margin_top(12);
    row.set_margin_bottom(12);

    let label = Label::new(Some("Output folder"));
    label.set_halign(Align::Start);
    label.add_css_class("section-label");

    let dir_btn = Button::new();
    dir_btn.add_css_class("flat");
    dir_btn.add_css_class("dir-button");

    let dir_label = Label::new(Some("~/Downloads/permutt/"));
    dir_label.set_halign(Align::Start);
    dir_label.set_ellipsize(gtk::pango::EllipsizeMode::Start);
    dir_btn.set_child(Some(&dir_label));

    let dir_label_ref = dir_label.clone();
    let state_dir = state.clone();

    dir_btn.connect_clicked(move |btn| {
        let dialog = FileDialog::builder().title("Choose output folder").build();

        let dir_label_inner = dir_label_ref.clone();
        let state_inner = state_dir.clone();
        let win = btn
            .root()
            .and_then(|r| r.downcast::<ApplicationWindow>().ok());

        dialog.select_folder(win.as_ref(), gtk::gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    dir_label_inner.set_text(&path.to_string_lossy());
                    if let Ok(mut s) = state_inner.lock() {
                        s.output_dir = Some(path);
                    }
                }
            }
        });
    });

    row.append(&label);
    row.append(&dir_btn);
    row
}

fn show_status(label: &Label, rev: &Revealer, msg: &str, ok: bool) {
    label.set_text(msg);
    if ok {
        label.remove_css_class("status-error");
        label.add_css_class("status-ok");
    } else {
        label.remove_css_class("status-ok");
        label.add_css_class("status-error");
    }
    rev.set_reveal_child(true);
}

fn default_output_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Downloads").join("permutt")
}
