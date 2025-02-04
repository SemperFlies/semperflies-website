use askama::Template;
use axum::extract::Path;
use axum::response::Html;
use std::collections::HashMap;
use tracing::warn;

use crate::components::carousel::{CarouselTemplate, HasCarousel, Image};

use crate::util::all_images_in_directory;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate {
    gear: HashMap<String, Vec<Gear>>,
}

pub async fn patrol_gear() -> Html<String> {
    let template = PatrolGearTemplate {
        gear: crate::database::builtins::builtin_gear(),
    };
    warn!("got gear template: {:?}", template);
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

impl HasCarousel for PatrolGearTemplate {}

#[derive(Debug)]
pub struct Gear {
    pub id: u32,
    pub price: u32,
    pub carousel: CarouselTemplate,
}

impl PatrolGearTemplate {
    fn tops(&self) -> &Vec<Gear> {
        self.gear.get(TOPS).unwrap()
    }
    fn hats(&self) -> &Vec<Gear> {
        self.gear.get(HATS).unwrap()
    }
    fn misc(&self) -> &Vec<Gear> {
        self.gear.get(MISC).unwrap()
    }
}

pub const TOPS: &str = "Tops";
pub const HATS: &str = "Hats";
pub const MISC: &str = "Miscellaneous";
