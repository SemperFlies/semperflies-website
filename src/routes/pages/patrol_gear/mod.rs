use std::collections::HashMap;

use askama::Template;
use axum::response::Html;
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
        gear: builtin_gear(),
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
    id: u32,
    price: u32,
    carousel: CarouselTemplate,
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
        if src.contains("sticker") {
            let images = vec![Image {
                src,
                alt: String::new(),
                subtitle: String::from("2 inches Tall and 3 ¾ Wide"),
            }];
            all_misc.push(Gear {
                id: 1,
                price: 5,
                carousel: CarouselTemplate {
                    images,
                    auto_scroll: false,
                    show_subtitles: true,
                },
            })
        } else if src.contains("patch") {
            let images = vec![Image {
                src,
                alt: String::new(),
                subtitle: String::from("2 ¾ inches round"),
            }];
            all_misc.push(Gear {
                id: 2,
                price: 10,
                carousel: CarouselTemplate {
                    images,
                    auto_scroll: false,
                    show_subtitles: true,
                },
            })
        } else {
            let images = vec![Image {
                src,
                alt: String::new(),
                subtitle: String::new(),
            }];
            all_misc.push(Gear {
                id: 3,
                price: 22,
                carousel: CarouselTemplate {
                    images,
                    auto_scroll: false,
                    show_subtitles: false,
                },
            })
        };
    }

    let all_tops_imgs: Vec<String> = all_images_in_directory(&tops_path)
        .unwrap()
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    let mut all_tops = vec![];
    let (hoodie_imgs, t_imgs): (Vec<String>, Vec<String>) = all_tops_imgs
        .into_iter()
        .partition(|src| src.contains("hoodie"));
    let images = hoodie_imgs
        .into_iter()
        .map(|src| Image {
            src,
            alt: String::new(),
            subtitle: String::new(),
        })
        .collect();
    all_tops.push(Gear {
        id: 1,
        price: 60,
        carousel: CarouselTemplate {
            images,
            auto_scroll: false,
            show_subtitles: false,
        },
    });
    let images = t_imgs
        .into_iter()
        .map(|src| Image {
            src,
            alt: String::new(),
            subtitle: String::new(),
        })
        .collect();

    all_tops.push(Gear {
        id: 2,
        price: 40,
        carousel: CarouselTemplate {
            images,
            auto_scroll: false,
            show_subtitles: false,
        },
    });

    let all_hats_imgs: Vec<String> = all_images_in_directory(&hats_path)
        .unwrap()
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();

    let mut all_hats = vec![];
    for src in all_hats_imgs {
        let price = 40;
        let id = match src.rsplit_once("/").unwrap().1 {
            "black-baseball.webp" => 1,
            "black-baseball2.webp" => 2,
            "black-beanie.webp" => 3,
            "camo-baseball.webp" => 4,
            "camo-flatbill.webp" => 5,
            "camo-flatbill2.webp" => 6,
            "camo-trucker.webp" => 7,
            "grey-flatbill.webp" => 8,
            "grey-flatbill2.webp" => 9,
            "grey-red-flatbill.webp" => 10,
            "white-beanie.webp" => 11,
            other => panic!("encountered unexpected hat img: {}", other),
        };

        let images = vec![Image {
            src,
            alt: String::new(),
            subtitle: String::new(),
        }];
        all_hats.push(Gear {
            id,
            price,
            carousel: CarouselTemplate {
                images,
                auto_scroll: false,
                show_subtitles: false,
            },
        });
    }

    all_hats.sort_by(|a, b| a.id.cmp(&b.id));

    let mut map = HashMap::new();
    map.insert(TOPS.to_string(), all_tops);
    map.insert(HATS.to_string(), all_hats);
    map.insert(MISC.to_string(), all_misc);
    map
}
