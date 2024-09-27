#[derive(Debug, Clone)]
pub enum Message {
    Watcher(notify::Event),
    Markdown,
}
