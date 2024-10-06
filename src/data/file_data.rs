#[derive(Debug)]
pub enum FileData {
    Svg(iced::widget::svg::Handle),
    Image(iced::widget::image::Handle),
    Markdown(Vec<iced::widget::markdown::Item>),
    Text(String),
}

impl Clone for FileData {
    fn clone(&self) -> Self {
        panic!("Shouldn't be cloning FileData")
    }
}
