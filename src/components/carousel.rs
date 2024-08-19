use askama::Template;

#[derive(Template, Debug)]
#[template(path = "components/carousel.html")]
pub struct CarouselTemplate {
    pub images: Vec<Image>,
}

#[derive(Debug)]
pub struct Image {
    src: String,
    alt: String,
}

impl Image {
    pub fn new(src: &str, alt: &str) -> Self {
        Self {
            src: src.to_owned(),
            alt: alt.to_owned(),
        }
    }
}
