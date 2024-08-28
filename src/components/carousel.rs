use askama::Template;

#[derive(Template, Debug, Clone)]
#[template(path = "components/carousel.html")]
pub struct CarouselTemplate {
    pub images: Vec<Image>,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub src: String,
    pub alt: String,
    pub subtitle: Option<String>,
}

pub trait HasCarousel {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}
