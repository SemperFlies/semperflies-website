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
            role: "Founder & Board Member".to_string(),
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
            image_url: "public/assets/images/board_members/jamie.webp".to_string(),
        },
        BoardMember {
            name: "Beverly L. Martin-Ornelas".to_string(),
            role: "Board Member".to_string(),
            description: r#"
            Beverly L. Martin comes to Semper Flies Foundation with a huge heart for Veterans. A
            United States Marine raised her, WWII &amp; Korean War Combat Veteran, my grandfather
            Verne Norman Martin, Jr. Beverly volunteers for Semper Flies Foundation as CFO with
            her financial expertise. Beverly owned &amp; operated one of the largest Trust Deed
            Investment Companies in Modesto, CA. for 32 years. She was “Businesswoman of the
            Year” in Modesto Ca promoted by The Modesto Bee. She raised three sons, all of whom
            went on to serve our country in the armed forces. Watching her father come back from
            WWII &amp; The Korean War and battle PTSD every day created her capacity to understand
            what Combat Veterans returning from theatre have to go through. She witnessed her
            father’s struggle with these conditions. Beverly was a massive part of Jamie Guajardo&#39;s
            treatment and remains a considerable part of his support network today. Along with Dan
            Ornelas Sr., Beverly’s husband, and our other board members, we feel that raising
            awareness for Veterans who struggle with PTSD &amp; TBI is a top priority.
            "#.to_string(),
            image_url: "public/assets/images/board_members/beverly.webp".to_string(),
        },
        BoardMember {
            name: "Dan Ornelas".to_string(),
            role: "Board Member".to_string(),
            description: r#"
            Dan Ornelas Sr. comes to Semper Flies Foundation with an understanding of the war
            plaguing our Veterans today. With Beverly, he raised three sons who served in the
            armed forces. Dan is the epitome of a Christian Man. He instilled in his sons the
            importance of walking with Jesus Christ. Dan is the CEO of a trust deed investment
            company today, MFS Preferred, that serves a private portfolio in the Modesto area. He
            also played a massive part in getting Jamie Guajardo the help he desperately needed
            after returning from Iraq. He is a significant part of Jamie’s support network today. Like
            Beverly, he is committed to raising awareness for Veterans who battle PTSD &amp; TBI.                
            "#.to_string(),
            image_url: "public/assets/images/board_members/dan.webp".to_string(),
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
