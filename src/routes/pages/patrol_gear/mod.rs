use std::collections::HashMap;

use askama::Template;
use axum::response::Html;

use super::util::all_images_in_directory;

#[derive(Template, Debug)]
#[template(path = "pages/patrol_gear.html")]
pub struct PatrolGearTemplate {
    gear: HashMap<String, Vec<Gear>>,
}

pub async fn patrol_gear() -> Html<String> {
    let template = PatrolGearTemplate {
        gear: builtin_gear(),
    };
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

#[derive(Debug)]
pub struct Gear {
    src: String,
    price: u32,
}

pub const TOPS: &str = "Tops";
pub const HATS: &str = "Hats";
pub const MISC: &str = "Miscellaneous";

fn builtin_gear() -> HashMap<String, Vec<Gear>> {
    let merch_path = "public/assets/images/merchandise";
    let hats_path = format!("{}/hats", merch_path);
    let misc_path = format!("{}/misc", merch_path);
    let tops_path = format!("{}/tops", merch_path);

    let all_misc_imgs: Vec<String> = all_images_in_directory(&misc_path)
        .unwrap()
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    let mut all_misc = vec![];
    for src in all_misc_imgs {
        let price = if src.contains("sticker") { 5 } else { 22 };
        all_misc.push(Gear { src, price });
    }

    let all_tops_imgs: Vec<String> = all_images_in_directory(&tops_path)
        .unwrap()
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    let mut all_tops = vec![];
    for src in all_tops_imgs {
        let price = if src.contains("hoodie") { 60 } else { 40 };
        all_tops.push(Gear { src, price });
    }

    let all_hats_imgs: Vec<String> = all_images_in_directory(&hats_path)
        .unwrap()
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();
    let mut all_hats = vec![];
    for src in all_hats_imgs {
        let price = 40;
        all_hats.push(Gear { src, price });
    }

    let mut map = HashMap::new();
    map.insert(TOPS.to_string(), all_tops);
    map.insert(HATS.to_string(), all_hats);
    map.insert(MISC.to_string(), all_misc);
    map
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
