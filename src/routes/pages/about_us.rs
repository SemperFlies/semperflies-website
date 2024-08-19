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
            description: "Our founder and veteran".to_string(),
            image_url: "public/assets/images/board_members/business.jpg".to_string(),
        },
        BoardMember {
            name: "Other Guajardo".to_string(),

            role: "Other Role".to_string(),
            description: "This is a board member".to_string(),
            image_url: "public/assets/images/board_members/business2.jpg".to_string(),
        },
        BoardMember {
            name: "Another Guajardo".to_string(),
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
