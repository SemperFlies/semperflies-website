use askama::Template;

use crate::database::models::DBImage;

#[derive(Template, Debug, Clone)]
#[template(path = "components/carousel.html")]
pub struct CarouselTemplate {
    pub images: Vec<Image>,
    pub auto_scroll: bool,
    pub show_subtitles: bool,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub src: String,
    pub alt: String,
    pub subtitle: String,
}

impl From<DBImage> for Image {
    fn from(value: DBImage) -> Self {
        Self {
            src: value.path,
            alt: value.alt,
            subtitle: value.subtitle.unwrap_or(String::new()),
        }
    }
}

pub trait HasCarousel {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}
