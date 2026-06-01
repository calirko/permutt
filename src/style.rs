use gtk::CssProvider;

const CSS: &str = r#"
.panel-title {
    font-size: 13px;
    font-weight: bold;
}

.section-label {
    font-size: 11px;
    opacity: 0.6;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.pill-button {
    border-radius: 20px;
    padding: 6px 16px;
    font-weight: 600;
}

.file-list row {
    border-radius: 6px;
    margin: 2px 8px;
    padding: 2px 0;
}

.format-badge {
    font-size: 10px;
    font-weight: 700;
    border-radius: 4px;
    padding: 1px 5px;
}

.drop-hint {
    opacity: 0.4;
}

.drop-icon {
    font-size: 40px;
}

.drop-primary {
    font-size: 15px;
    font-weight: 600;
}

.drop-secondary {
    font-size: 12px;
}

.drop-overlay {
    border: 2px dashed transparent;
    border-radius: 8px;
    margin: 8px;
}

.dir-button {
    border-radius: 6px;
    padding: 4px 8px;
}

.format-chip {
    border-radius: 20px;
    padding: 4px 12px;
    font-size: 12px;
    font-weight: 600;
}

.status-label {
    font-size: 12px;
    margin-bottom: 8px;
    padding: 6px 0;
}

.status-ok {
    color: green;
}

.status-error {
    color: red;
}

separator {
    min-height: 1px;
}
"#;

pub fn load() {
    let provider = CssProvider::new();
    provider.load_from_string(CSS);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("could not connect to display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
