use std::{collections::HashMap, path::PathBuf};

use askama::Template;
use axum::response::Html;

use crate::components::carousel::{self, CarouselTemplate, HasCarousel, Image};

#[derive(Template, Debug)]
#[template(path = "pages/landing.html")]
pub struct LandingTemplate {
    carousel: CarouselTemplate,
}

impl HasCarousel for LandingTemplate {}
pub async fn landing() -> Html<String> {
    let paths = super::util::all_images_in_directory("public/assets/images/landing_page").unwrap();
    let images = paths_to_ordered_images(paths);
    let carousel = CarouselTemplate {
        show_subtitles: true,
        images,
        auto_scroll: true,
    };
    let template = LandingTemplate { carousel };
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

fn paths_to_ordered_images(paths: Vec<PathBuf>) -> Vec<Image> {
    let mut map = subtitle_alt_map();
    let mut images: Vec<(usize, Image)> = vec![];
    for path in paths {
        let path_str = path.to_str().unwrap();
        let src = path_str.to_owned();
        let name = src.rsplit_once('.').unwrap().0.rsplit_once('/').unwrap().1;
        if let Some((idx, alt, subtitle)) = map.remove(name) {
            let subtitle = subtitle.replace('\n', "<br />");
            images.push((idx, Image { src, alt, subtitle }))
        }
    }
    images.sort_by(|a, b| a.0.cmp(&b.0));
    let images = images.into_iter().map(|(_, i)| i).collect();
    images
}

fn subtitle_alt_map() -> HashMap<String, (usize, String, String)> {
    let mut map = HashMap::new();
    let mut ordering_idx = 0;
    map.insert(
        "scout-snipers".to_string(),
        (
            ordering_idx,
            "A photo of multiple soldiers in camo".to_string(),
            r#"
1997
3rd Bn 4th Marines
STA Platoon Scout Snipers 
Six members of this platoon would go on to operate in Iraq. 
                "#
            .to_string(),
        ),
    );
    ordering_idx += 1;

    map.insert(
        "team".to_string(),
        (
            ordering_idx,
            "A black and white photo of multiple soldiers".to_string(),
            r#"
Green Side Patrolling
Team leader #2 5th Platoon
1st Force Reconnaissance Co.
                "#
            .to_string(),
        ),
    );
    ordering_idx += 1;

    map.insert(
        "wetworks".to_string(),
        (
            ordering_idx,
            "A photo of soldiers in diving gear".to_string(),
            r#"
Diving Operations
Team leader #2 5th Platoon
1st Force Reconnaissance Co.
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;

    map.insert(
        "kuwait".to_string(),
        (
            ordering_idx,
            "A photo of a soldier in front of his gear".to_string(),
            r#"
Kuwait, Camp Commando
Platoon Inspection
Direction Action Raid Safwan Hill, Iraq
SSgt Guajardo
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;

    map.insert(
        "digging".to_string(),
        (
            ordering_idx,
            "A photo of a soldier posing with a shovel".to_string(),
            r#"
Digging In
Direct Action Raid
Safwan Hill, Iraq
D-8
March 20, 2003
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "jamie-and-flag".to_string(),
        (
            ordering_idx,
            "soldiers posing with a flag in Baghdad, Iraq".to_string(),
            r#"
March 20, 2003
Iraq Invasion 
Safwan Hill. 
1st Force Reconnaissance Co. 5th Platoon
SSgt Guajardo, Team Leader, Team #2
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "baghdad-burning".to_string(),
        (
            ordering_idx,
            "photo of a burning building".to_string(),
            r#"
Baghdad, Iraq
5th Platoon on patrol
1st Force Reconnaissance Company.
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "baghdad-cash".to_string(),
        (
            ordering_idx,
            "a picture of soldiers with a bunch of cash".to_string(),
            r#"
Baghdad, Iraq
Sitting on pile of cash. 
Recovered during CQB/Bank Hits. 
5th Platoon
1st Force Reconnaissance Company.
Baghdad SWAT
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "tv-blurb".to_string(),
        (
            ordering_idx,
            "a screenshot of a news broadcast with two soldiers holding M4 assault rifles"
                .to_string(),
            r#"
Operating in Baghdad, Iraq. 
2003
SSgt Guajardo
Sgt Anderson
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;

    map.insert(
        "article".to_string(),
        (
            ordering_idx,
            "A time magazine article with an american soldier subdueing a man".to_string(),
            r#"
Time Magazine
April 28, 2003 Edition
SSgt Guajardo J.M. 
Operating in Baghdad, Iraq.
        "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "baghdad".to_string(),
        (
            ordering_idx,
            "soldiers posing with a flag in Baghdad, Iraq".to_string(),
            r#"
April, 2003
Baghdad, Iraq
1st Force Recon 5th Platoon 
This platoon would earn the name “Baghdad SWAT” for their operations. 
                "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "medal".to_string(),
        (
            ordering_idx,
            "A navy commendation medal".to_string(),
            r#"
General Conway & Jamie Guajardo
Babylon, Iraq
Retrograde back to Kuwait
Navy Commendation Medal 
“V” for Valor
"#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "with-rabbit".to_string(),
        (
            ordering_idx,
            "an image of a man holding a rabbit".to_string(),
            r#"
Jamie Guajardo
Veterans Administration PTSD Hospital 
Denver, Colorado 2019
"#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "first-flies".to_string(),
        (
            ordering_idx,
            "a how to tie a flie diagram".to_string(),
            r#"
Semper Flies was born in a residential PTSD treatment facility in Colorado.
These are the 1st flys Jamie ever made."#
                .to_string(),
        ),
    );

    ordering_idx += 1;

    map.insert(
        "flies".to_string(),
        (
            ordering_idx,
            "an image of a lot of flies for fly fishing".to_string(),
            r#"
The first 22 Semper Flies ever made.
22 Veterans a day lose their life to Veteran Suicide.
            "#
            .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "mileo".to_string(),
        (
            ordering_idx,
            "An image of cards, including the veteran crisis line and a semperflies business card. As well as a photo of a solider".to_string(),
                r#"
Semper Flies Foundation
"# .to_string(),
        ),
    );

    ordering_idx += 1;
    map.insert(
        "jamie-with-truck".to_string(),
        (
            ordering_idx,
            "an image of a tattooed man in front of a branded truck".to_string(),
            r#"
Jamie Guajardo
Semper Flies Foundation Mobile HQ
Battling PTSD & TBI everyday is ubiquitous & unrelenting. 
It can break you or make you. 
It is from these platforms/conditions that we can either fall or rise. 
I choose the latter. 
Choose the latter with me and “Stay, The Fight!”
"#
            .to_string(),
        ),
    );

    map
}
