use askama::Template;
use axum::response::Html;

#[derive(Debug)]
struct BoardMember {
    image_url: String,
    name: String,
    role: String,
    description: String,
}

fn board_members() -> Vec<BoardMember> {
    vec![
        BoardMember {
            name: "Jamie Guajardo".to_string(),
            role: "Founder".to_string(),
            description: r#"
            In 2015, after battling Post Traumatic Stress Disorder & Traumatic Brain Injury for twelve years without seeking help he finally walked through the doors of a Veterans Administration Hospital for the first time.
            <br/>
            <br/>
            In 2019, the severity of his PTSD and TBI eventually resulted in his stay at a residential Veterans Administration PTSD Hospital in Colorado where he worked with the best doctors in the nation.
            To this day Jamie continues to struggle daily, but through years of gathering knowledge he has made a decision to save his life, and as many other Veterans as he possibly can.
            <br/>
            <br/>
            During his stay at the residential treatment hospital in Colorado, Jamie learned how to tie fishing flys as a form of “grounding.” He enjoyed this project and vowed to himself
            that when he got out he was going to create a platform called Semper Flies and use it to raise awareness for Veterans that struggle with PTSD & TBI, and that’s exactly what he did.
            "#.to_string(),
            image_url: "public/assets/images/board_members/jamie.jpeg".to_string(),
        },
        BoardMember {
            name: "Beverly Guajardo".to_string(),
            role: "Other Role".to_string(),
            description: "This is a board member".to_string(),
            image_url: "public/assets/images/board_members/business2.jpg".to_string(),
        },
        BoardMember {
            name: "Dan Guajardo".to_string(),
            role: "Another Role".to_string(),
            description: "This is a board member".to_string(),
            image_url: "public/assets/images/board_members/old.jpg".to_string(),
        },
    ]
}

#[derive(Template, Debug)]
#[template(path = "pages/about_us.html")]
pub struct AboutUsTemplate {
    board_members: Vec<BoardMember>,
}

pub async fn about_us() -> Html<String> {
    let tmpl = AboutUsTemplate {
        board_members: board_members(),
    };
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
