use askama::Template;
use axum::response::Html;
use rand::prelude::*;

#[derive(Template, Debug)]
#[template(path = "pages/support.html")]
pub struct SupportTemplate {
    resources: Vec<SupportResource>,
}

#[derive(Debug)]
struct Address {
    line_2: Option<String>,
    line_1: String,
    city: String,
    state: String,
    zip: String,
}

#[derive(Debug)]
struct SupportResource {
    name: String,
    description: String,
    missions: Vec<String>,
    phone: Option<String>,
    email: Option<String>,
    physical_address: Option<Address>,
}

pub async fn support() -> Html<String> {
    let template = SupportTemplate {
        resources: generate_support_resources(),
    };
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

fn generate_support_resources() -> Vec<SupportResource> {
    let mut resources = Vec::new();

    let names = vec![
        "Red Cross",
        "Salvation Army",
        "Habitat for Humanity",
        "United Way",
        "Feeding America",
    ];

    let descriptions = vec![
        "Providing emergency assistance and disaster relief.",
        "Offering shelter, food, and social services.",
        "Building affordable housing and revitalizing communities.",
        "Supporting health, education, and financial stability programs.",
        "Fighting hunger and distributing food to those in need.",
    ];

    let missions = vec![
        vec!["Disaster Relief", "Blood Donation", "Health Services"],
        vec!["Homeless Services", "Rehabilitation", "Youth Programs"],
        vec![
            "Affordable Housing",
            "Home Repairs",
            "Neighborhood Revitalization",
        ],
        vec!["Education", "Income", "Health"],
        vec!["Food Pantries", "Meal Programs", "Nutrition Education"],
    ];

    let mut rng = rand::thread_rng();

    for _ in 0..5 {
        let name = names.choose(&mut rng).unwrap();
        let description = descriptions.choose(&mut rng).unwrap();
        let missions = missions
            .choose(&mut rng)
            .unwrap()
            .iter()
            .map(|m| m.to_string())
            .collect();
        let phone = if rng.gen_bool(0.7) {
            Some(format!(
                "1-800-{:03}-{:04}",
                rng.gen_range(100..999),
                rng.gen_range(1000..9999)
            ))
        } else {
            None
        };
        let email = if rng.gen_bool(0.8) {
            Some(format!(
                "info@{}.org",
                name.replace(" ", "_").to_lowercase()
            ))
        } else {
            None
        };
        let physical_address = if rng.gen_bool(0.6) {
            Some(Address {
                line_1: format!(
                    "{} Something {}",
                    rng.gen_range(100..999),
                    ["St", "Ave", "Blvd", "Rd"].choose(&mut rng).unwrap()
                ),
                line_2: None,
                city: ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                state: ["NY", "CA", "IL", "TX", "AZ"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                zip: format!("{:05}", rng.gen_range(10000..99999)),
            })
        } else {
            None
        };

        resources.push(SupportResource {
            name: name.to_string(),
            description: description.to_string(),
            missions,
            phone,
            email,
            physical_address,
        });
    }

    resources
}
