use super::models::DBTestimonial;
use crate::{
    components::carousel::{CarouselTemplate, Image},
    routes::pages::{
        dedications::Dedication,
        patrol_log::logs::Log,
        support::{Address, SupportResource},
    },
    util::all_images_in_directory,
};
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

/// This module contains all 'builtin' database data
/// This is done this way (rather got_jmi(than through a migration file) for the sake of brevity.
/// The alternative would be to manually create UUIDs for all images & entries and put that into
/// a .sql migration file
///
/// The biggest downside of this method is that these entries *cannot* be removed
/// or edited by Jamie, a developer has to come to this file in order to adjust them.

pub fn builtin_dedications() -> Vec<Dedication> {
    let mileo = Dedication {
        id: Uuid::new_v4(),
        names:vec![ "Corporal Jason David Mileo".to_string()],
        birth: NaiveDate::from_ymd_opt(1982, 12, 14).unwrap(),
        death: NaiveDate::from_ymd_opt(2003, 4, 14).unwrap(),
        bio: r#"Corporal Jason David Mileo deployed to Iraq with 3rd Battalion 4th Marines in 2003. He fought along side his Marine Brothers during the Shock-N-Awe, the push on Baghdad, and he was in the city square when the statue of Saddam Hussein fell.
<br/>
<br/>
On April 14, 2003, Corporal Mileo bravely crawled into an elevated position on a night patrol so he could provide security over watch for his Marines. They were on a movement to contact patrol and had departed friendly lines with one thing in mind; contact. That evening there was an elevation in activity. Gunfire was being exchanged directly outside the walls of the Marines fortified position in downtown Baghdad. The gun fire continued intermittently throughout the late afternoon and into the dusk of night. Marine Scout Snipers (8541’s) from an elite unit were manning the most elevated position of the Marines stronghold. “The tragic death of Corporal Mileo was the result of several significant breakdowns in discipline, coordination and communication that set the stage for this horrific incident”.
<br/>
-Maj. Gen. J.N. Mattis, commander of the 1st Marine Division.
<br/>
<br/>
General Mattis also wrote:
<br/>
“Even though no one event or person was the catalyst for Corporal Mileo's death, one break in the chain of events may have spared his life." That night, Corporal Mileo was tragically mistaken for an enemy fighter and engaged by that Marine Scout Sniper Team. Everyone was doing what they were trained to do; believing he was an enemy target preparing a rooftop position, the snipers shot and killed him. “The devastation on the faces of every Marine that was present at his memorial the following morning can never be embodied in words. I’ve wished I can go back and say something, or I think I did.. I don’t remember. One second the memory is clear, the next it’s blank. But the faces, the faces of his Marine Brothers.. those will be burned into my mind. This moment redefined my entire life. The loss of that Warrior will have catastrophic effects on me for the rest of my life. I’ll never be able to leave that rooftop in my mind; life sentence.” -Marine Scout Sniper
<br/>
(Spotter/Jamie Martin Guajardo)
 "#.to_string(),
        carousel:CarouselTemplate { images: vec![Image {
            src: "public/assets/images/dedications/mileo.webp".to_string(),
            alt: "An image of a soldier".to_string(),
            subtitle: "".to_string(),
        }], auto_scroll: false, show_subtitles: false }
    };

    let fifth_platoon = Dedication {
        id: Uuid::new_v4(),
        names: vec![
            "SSgt Vincent Sabasteanski".to_string(),
            "SSgt David Galloway".to_string(),
            "SSgt Jeffrey Starling".to_string(),
            "Cpl Mark Baca".to_string(),
            "HM1 Jay Asis".to_string(),
            "GySgt James Paige".to_string(),
            "SSgt William Dame".to_string(),
        ],
        birth: NaiveDate::from_ymd_opt(1775, 11, 10).unwrap(),
        death: NaiveDate::from_ymd_opt(1999, 12, 9).unwrap(),
        bio: r#"On December 9, 1999 1st Force Reconnaissance Company suffered a major loss. A CH-46 was carrying 5th Platoon for a V.B.S.S (Visit Board Search Seizure). As the helicopter made the approach to the USNS Pecos the piolet became tangled in the netting causing it to flip upside down into the Pacific Ocean off the coast of Point Loma, Ca. This was a joint operation with the Navy SEALS. The SEALS had safety boats in the water and were able to rescue eleven survivors. The seven Warriors that lost their life’s that day paid the ultimate sacrifice in defense of our country. I still communicate with family of the fallen warriors. As a platoon we suffered mentally together and individually forever. The wives of the fallen Warriors showed us unmeasurable strength. Huge “Thank You” to the Navy SEALS for being so tactically proficient and bringing our Brothers aboard in the time of crisis."#.to_string(),
        carousel: CarouselTemplate {
            images: vec![Image {
                src: "public/assets/images/dedications/5th_platoon.webp".to_string(),
                alt: "a dedication to multiple solidiers".to_string(),
                subtitle: "".to_string(),
            }],
            auto_scroll: false,
            show_subtitles: false,
        },
    };

    let maxwell = Dedication {
        id: Uuid::new_v4(),
        names: vec![
            "Sergeant Jason Maxwel".to_string()
        ],
        birth: NaiveDate::from_ymd_opt(1978, 03, 04).unwrap(),
        death: NaiveDate::from_ymd_opt(2003, 10, 30).unwrap(),
        bio: r#"Sergeant Jason Maxwell was all heart & the epitome of a Force Recon Marine. I met him when we were standing by for the Iraq Invasion at Camp Commando in Kuwait. Our GP (general purpose) platoon size tents were right next to each other. Our platoons spent a lot of time together; it’s a small community anyway so a lot of us knew each other as a result from time in the unit. They were deployed to Iraq from Kāné Ohe Bay, Hawai’i, 4th Force Reconnaissance Co. Our platoon was out of Camp Pendleton, Ca, 1st Force Reconnaissance Co. After Combat Operations in Iraq our platoons returned back to our respective bases. I went out to Yuma, Az. to be an instructor at the Military Free Fall School, H.A.L.O. Shortly after becoming an instructor I looked up one day with the biggest smile and it was returned as Maxwell walked through the door to be a student. It was great to see him again; like I said small community. Maxwell did great progressing through the course. Another Force Recon Marine and myself were his instructors. Maxwell lost his life training to defend this country, an already accomplished Combat Veteran. He had a full malfunction on his parachute and left this life way too early. He will never be forgotten. 
RIP Warrior. 
S/F. 
ML&R. "#.to_string(),
        carousel: CarouselTemplate {
            images: vec![Image {
                src: "public/assets/images/dedications/maxwell.webp".to_string(),
                alt: "a dedication to maxwell".to_string(),
                subtitle: "".to_string(),
            }],
            auto_scroll: false,
            show_subtitles: false,
        },
    };

    let sam_spicer = Dedication {
        id: Uuid::new_v4(),
        names: vec![
            "Sam Spicer".to_string()
        ],
        birth: NaiveDate::from_ymd_opt(1775, 11, 10).unwrap(),
        death: NaiveDate::from_ymd_opt(2016, 01, 09).unwrap(),
        bio: r#" Honoring Sam Spicer (Gary). Another Marine taken way too early from the earth; January 9th, 2016. We were Snipers in the same platoon, 3/4 STA. 
He was the kind of Marine that made everything look easy. He quickly earned a spot to Sniper School and graduated with ease. Quickly back to the platoon increasing mission capability operating as an 8541.
Marine Scout Sniper. Stellar Marine. Thank you for your service and sacrifice, Brother. Semper Fi.
 ⚡️⚡"#.to_string(),
        carousel: CarouselTemplate {
            images: vec![Image {
                src: "public/assets/images/dedications/sam_spicer.webp".to_string(),
                alt: "a dedication to Sam Spicer".to_string(),
                subtitle: "".to_string(),
            }],
            auto_scroll: false,
            show_subtitles: false,
        },
    };
    vec![mileo, fifth_platoon, maxwell, sam_spicer]
}

pub fn builtin_logs() -> Vec<Log> {
    let images = all_images_in_directory("public/assets/images/patrol_log/fishing_trip").unwrap();
    let images = images
        .into_iter()
        .map(|path| Image {
            src: path.to_str().unwrap().to_string(),
            alt: String::new(),
            subtitle: String::new(),
        })
        .collect();

    let carousel = CarouselTemplate {
        show_subtitles: false,
        images,
        auto_scroll: false,
    };
    let fishing_trip = Log {
        id: Uuid::new_v4(),
        heading: "Semperflies Fishing Trip".to_string(),
        description: "Semper Flies Foundation & Tahoe Fly Fishing Outfitters teamed up to send (2) Combat Veterans on a fly fishing trip they would remember for the rest of their lives.".to_string(),
        date: NaiveDate::from_ymd_opt(2023, 06, 21).unwrap(),
        carousel,
    };
    vec![fishing_trip]
}

pub fn builtin_support_resources() -> Vec<SupportResource> {
    let motivational_marine = SupportResource{
        id: Uuid::new_v4(),
        name: "The Motivational Marine".to_string(),
        description: r#"The Motivational Marine is dedicated to empowering individuals to break free from the confines of their minds and fully engage with their lives. 
Using evidence-based knowledge, we provide insightful coaching that reveals the often-overlooked aspects of how our minds work. 
Understanding is the first step to improvement—because you can't change what you don't know exists. 
Our mission is to illuminate these hidden facets, enabling you to live with intention, purpose, and clarity.
        "#.to_string(),
        phone: Some("(260)-466-8929".to_string()),
        facebook: Some("https://www.facebook.com/themotivationalmarine?mibextid=LQQJ4d".to_string()),
        linkedin: Some("https://www.linkedin.com/in/briangagye?utm_source=share&utm_campaign=share_via&utm_content=profile&utm_medium=ios_app".to_string()),
    logo: Some(Image {
            src: "public/assets/images/support/motivational_marine.webp".to_string(),
            alt: "the motivational marine logo".to_string(),
            subtitle: String::new(),
        }),
        email: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        physical_address: None,
        website_url: None,

    };

    let mission_22 = SupportResource {
        id: Uuid::new_v4(),
        name: "Mission 22".to_string(),
        description: r#"Mission 22 provides support to Veterans and their families when they need it most: right now. Through a comprehensive approach of outreach, events, and programs, we’re promoting long-term wellness and sustainable growth."#.to_string(),
        physical_address: Some(Address{
                line_2: Some("#910".to_string()),
                line_1: "649 N Larch St".to_string(),
                city: "Sisters".to_string(),
                state: "OR".to_string(),
                zip: "97759".to_string(),
            }),
        phone: Some("(503)-908-8505".to_string()),
        website_url: Some("https://mission22.com/".to_string()),
        logo: Some(Image {
            src: "public/assets/images/support/mission_22.webp".to_string(),
            alt: "the mission 22 logo".to_string(),
            subtitle: String::new(),
        }),
    linkedin: None,
        email: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        facebook: None,
    };

    let reconnaissance_foundataion = SupportResource {
        id: Uuid::new_v4(),
        name: "Marine Reconnaissance Foundation".to_string(),
        description: r#"The Marine Reconnaissance Foundation (MRF) is committed to serving the Marine Reconnaissance Community by providing support to active-duty, retired and former teammates via reoccurring annual and emergency support programs for Reconnaissance Marines, and Special Amphibious Reconnaissance Corpsmen (SARC) deployed and our families."#.to_string(),
        physical_address: Some(Address {
                line_2: None,
                line_1: "91-1000 Hoomanao St".to_string(),
                city: "Ewa Beach".to_string(),
                state: "HI".to_string(),
                zip: "96706".to_string(),
            }),
        phone: Some("(808)-690-7025".to_string()),
        email: Some("info@reconfoundation.org".to_string()),
        website_url: Some("https://reconfoundation.org/".to_string()),
        logo: Some(Image {
            src: "public/assets/images/support/marine-recon-foundation-logo.webp".to_string(),
            alt: "the marine recon foundation logo".to_string(),
            subtitle: String::new(),
        }),
    linkedin: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        facebook: None,
    };

    let ltffo = SupportResource {
        id: Uuid::new_v4(),
        name: "Lake Tahoe Fly Fishing Outfitters".to_string(),
        description: r#"Tahoe Fly Fishing Outfitters was an integral part of getting Semper Flies Foundation started. I source my materials here and received advice & coaching for the first Semper Flie ever made. In addition, they are a huge supporter of Veterans. Located on the south shore of Lake Tahoe offering the most complete fly-fishing outfitter and shop for all things fly fishing in the Sierra. They offer private and group guided fishing trips. And, they have all the gear available at their shop for rent or purchase."#.to_string(),
        physical_address: Some(Address {
                line_2: None,
                line_1: "2705 Lake Tahoe Blvd.".to_string(),
                city: "South Lake Tahoe".to_string(),
                state: "CA".to_string(),
                zip: "96150".to_string(),
            }),
        phone: Some("(530) 541-8208".to_string()),
        website_url: Some("https://tahoeflyfishing.com/".to_string()),
        email: None,
        logo: Some(Image {
            src: "public/assets/images/support/ltffo.webp".to_string(),
            alt: "the Lake Tahoe Fly Fishing Outfitters logo".to_string(),
            subtitle: String::new(),
        }),
    linkedin: None,
        instagram: None,
        missions: vec![],
        twitter: None,
        threads: None,
        youtube: None,
        facebook: None,
    };

    vec![
        motivational_marine,
        mission_22,
        reconnaissance_foundataion,
        ltffo,
    ]
}

pub fn builtin_testimonials() -> Vec<DBTestimonial> {
    let jose_garcia = DBTestimonial {
        id: Uuid::new_v4(),
        firstname: "Jose".to_owned(),
        lastname: "Garcia".to_owned(),
        bio: None,
        content: r#"
I was graciously invited to attend a fly-fishing outing with a good Marine friend of mine.  All expenses were paid, and we would spend the day learning the ropes on fly fishing.  How could I say no?
<br />
<br />
We headed out to Lake Tahoe where I met Jamie Guajardo who gave us instruction on what we would be doing on our trip. I was completely surprised that Jamie, of Semper Flies Foundation, was not going to be coming with us seeing that he had arranged this entire trip through Tahoe Fly Fishing Outfitters. Thankful is not enough of a word for Jamie.  
<br />
<br />
I have been having some real bad mental health issues recently and figured that maybe this is what I needed.  And, I am glad I went. The escape from the city and just being out in the peacefulness of God's nature literally made me forget about my problems.  I spent the day learning how to fly fish with our guide, from Tahoe Fly Fishing outfitters, who was deeply knowledgeable and patient with me.  To top it off I caught a fish toward the end of the day.  
<br />
<br />
Being out there in the middle of nowhere, with the only sounds being of birds and the river water, made me forget about my problems and worries. It centered me for the day.  I am grateful for the opportunity to have attended this awesome trip and I am grateful for all involved, Jamie of Semper Flies Foundation, Tahoe Fly Fishing Outfitters and everyone else that made this day possible.  
<br />
<br />
Thank you and Semper Fidelis!"#.to_owned(),
    };

    let lawrence_turner = DBTestimonial {
        id: Uuid::new_v4(),
        firstname: "Lawrence".to_owned(),
        lastname: "Turner".to_owned(),
        bio: None,
        content: r#"
        To whoever is out there thinking of trying the fishing trip with Semper Flies and Lake Tahoe Fly Fishing, I
highly recommend.
<br/>
<br/>
Some of us Veterans have experienced unfathomable things overseas that live with us day in and day
out that are unexplainable that would just not make sense, if we attempted to put into words.
<br/>
<br/>
Jamie Guajardo is a Special Forces Marine, we did not serve together but chewed the same dirt at the
same time, he is a Giant! And a special person trying to heal his brothers.
<br/>
<br/>
Long story short, Jamie knows what it is like to have the feeling that lives with us. It’s an amazing thing
what he is doing for us on this level to try to heal.
Jamie set us up with South Lake Tahoe Fly Fishing for a beautiful day outdoors to help heal and figure
out our damage and wounds.
<br/>
<br/>
Started our day off at the shop where all of the staff were Awesome! Headed out to the river where we
trekked in about a 15-20 min ride on a brand new side by side with amazing views on the way in,
beautiful water, and unspeakable experience.
<br/>
<br/>
Our guide was very knowledgeable, patient, and put us on fun fighting fish! I really appreciate what
Jamie and South Lake Tahoe Fly Fishing Shop are doing for Veterans. It’s one step closer to normalcy!
Lol!
It’s only for a day please go drop a line with Semper Flies and South Lake Tahoe Fly Fishing Shop!
<br/>
<br/>
Semper Fidelis!
<br/>
3/5 Kilo Co.
<br/>
Phantom Fury.
        "#.to_owned(),
    };
    vec![jose_garcia, lawrence_turner]
}
