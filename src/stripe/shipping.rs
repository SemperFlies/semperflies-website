use std::{collections::HashMap, str::FromStr};

use super::products::{get_cached_products, Product};

/// How many boxes of each size are required
#[derive(Debug, PartialEq)]
pub enum ShippingInfo {
    Box { md: usize, lg: usize },
    Single(ShippingSize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippingSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug)]
struct ProductPacking {
    prod: Product,
    amt: usize,
}

pub struct ShippingIds {
    pub lg_box: String,
    pub md_box: String,
    pub large_item: String,
    pub medium_item: String,
    pub small_item: String,
}

impl Default for ShippingIds {
    fn default() -> Self {
        dotenv::dotenv().ok();
        Self {
            lg_box: std::env::var("LG_BOX").expect("NO LG_BOX env var"),
            md_box: std::env::var("MD_BOX").expect("NO MD_BOX env var"),
            large_item: std::env::var("LARGE_ITEM").expect("NO LARGE_ITEM env var"),
            medium_item: std::env::var("MEDIUM_ITEM").expect("NO MEDIUM_ITEM env var"),
            small_item: std::env::var("SMALL_ITEM").expect("NO SMALL_ITEM env var"),
        }
    }
}

impl ShippingInfo {
    /// Because shipping costs change based on the amount of items ordered, shipping costs are in the stripe system as
    /// products rather than shippings rates
    pub fn to_shipping_line_items(&self) -> Vec<stripe::CreateCheckoutSessionLineItems> {
        let shipping_ids = ShippingIds::default();
        match self {
            Self::Box { md, lg } => {
                let md_box_price = match get_cached_products()
                    .get(
                        &stripe::ProductId::from_str(&shipping_ids.md_box)
                            .expect("could not convert md box id to stripe::ProductId?"),
                    )
                    .expect("NO medium box product?")
                    .default_price
                    .as_ref()
                    .expect("no default price for medium box product?")
                {
                    stripe::Expandable::Id(id) => id.to_string(),
                    stripe::Expandable::Object(obj) => obj.id.to_string(),
                };
                let lg_box_price = match get_cached_products()
                    .get(
                        &stripe::ProductId::from_str(&shipping_ids.lg_box)
                            .expect("could not convert lg box id to stripe::ProductId?"),
                    )
                    .expect("NO large box product?")
                    .default_price
                    .as_ref()
                    .expect("No default price for large box product?")
                {
                    stripe::Expandable::Id(id) => id.to_string(),
                    stripe::Expandable::Object(obj) => obj.id.to_string(),
                };
                let mut all = vec![];
                if *md > 0 {
                    all.push(stripe::CreateCheckoutSessionLineItems {
                        quantity: Some(*md as u64),
                        price: Some(md_box_price),
                        ..Default::default()
                    })
                }
                if *lg > 0 {
                    all.push(stripe::CreateCheckoutSessionLineItems {
                        quantity: Some(*lg as u64),
                        price: Some(lg_box_price),
                        ..Default::default()
                    })
                }
                all
            }
            Self::Single(size) => {
                let id = match size {
                    ShippingSize::Small => shipping_ids.small_item,
                    ShippingSize::Medium => shipping_ids.medium_item,
                    ShippingSize::Large => shipping_ids.large_item,
                };

                let price = match get_cached_products()
                    .get(
                        &stripe::ProductId::from_str(&id)
                            .expect("could not convert id to stripe::ProductId?"),
                    )
                    .expect("NO single item shipping product?")
                    .default_price
                    .as_ref()
                    .expect("No default price for single product shipping?")
                {
                    stripe::Expandable::Id(id) => id.to_string(),
                    stripe::Expandable::Object(obj) => obj.id.to_string(),
                };
                vec![stripe::CreateCheckoutSessionLineItems {
                    quantity: Some(1),
                    price: Some(price),
                    ..Default::default()
                }]
            }
        }
    }
}

#[tracing::instrument(name = "get shipping info")]
pub fn get_shipping_info(items: &HashMap<stripe::ProductId, usize>) -> ShippingInfo {
    let cached = get_cached_products();
    let mut total_pct_md = 0.;
    let mut total_pct_lg = 0.;
    let mut prods = HashMap::new();

    if items.len() == 1 && items.values().next().is_some_and(|v| *v == 1) {
        let k = items.keys().next().unwrap();
        let prod = Product::try_from(
            cached
                .get(k)
                .cloned()
                .expect("didnt get product from cached products?"),
        )
        .expect("couldn't get internal product from stripe product");
        tracing::debug!("only one item ordered");
        return ShippingInfo::Single(prod.size_info.shipping_size);
    }

    let (mut any_item_exceeds_max_percent_md, mut any_item_exceeds_max_percent_lg) = (false, false);
    for (id, v) in cached.into_iter() {
        if let (Some(p), Some(amt)) = (Product::try_from(v).ok(), items.get(&id)) {
            let percent_md = ((*amt as f32 / p.size_info.medium as f32) * 100.)
                * (p.size_info.max_percent.unwrap_or(100) as f32 / 100.);
            let percent_lg = ((*amt as f32 / p.size_info.large as f32) * 100.)
                * (p.size_info.max_percent.unwrap_or(100) as f32 / 100.);
            tracing::debug!(
                "product: {:#?}\namt: {amt}\npct medium: {percent_md}\n pct large: {percent_lg}\n",
                p.info.name
            );
            if !any_item_exceeds_max_percent_md
                && p.size_info.max_percent.is_some_and(|v| {
                    tracing::debug!("v: {v}\npercent_md: {percent_md}");
                    v < percent_md as u32
                })
            {
                tracing::debug!("exceed md");
                any_item_exceeds_max_percent_md = true;
            }
            if !any_item_exceeds_max_percent_lg
                && p.size_info
                    .max_percent
                    .is_some_and(|v| v < percent_lg as u32)
            {
                tracing::debug!("exceed large");
                any_item_exceeds_max_percent_lg = true;
            }
            total_pct_md += percent_md;
            total_pct_lg += percent_lg;
            let packing = ProductPacking {
                prod: p,
                amt: amt.to_owned(),
            };
            prods.insert(id, packing);
        }
    }
    tracing::debug!("total pct medium: {total_pct_md}\ntotal pct large: {total_pct_lg}\n");

    if total_pct_md <= 100. && !any_item_exceeds_max_percent_md {
        tracing::debug!("returning md early");
        return ShippingInfo::Box { md: 1, lg: 0 };
    } else if total_pct_lg <= 100. && !any_item_exceeds_max_percent_lg {
        tracing::debug!("returning lg early");
        return ShippingInfo::Box { md: 0, lg: 1 };
    }

    let mut md = 0;
    let mut lg = 0;
    let mut prods_to_remove = vec![];

    while !prods.is_empty() {
        let mut box_filled_pct: f32 = 0.;
        let large_box =
            // total_pct_lg >= 100. &&
             total_pct_md >= 100.;
        if large_box {
            tracing::debug!("FILLING LARGE");
        } else {
            tracing::debug!("FILLING MEDIUM");
        }

        let amt_prods = prods.len();
        for (i, (id, prod_packing)) in prods.iter_mut().enumerate() {
            if prod_packing.prod.size_info.max_percent.is_some_and(|v| {
                tracing::debug!("this product has a limit of {v}%");
                v <= box_filled_pct as u32
            }) {
                tracing::debug!("no more of this product can be added to the box");
                if i + 1 == amt_prods {
                    box_filled_pct = 100.1;
                } else {
                    continue;
                }
            }

            let pct_lg = ((1. / prod_packing.prod.size_info.large as f32) * 100.)
                * (prod_packing.prod.size_info.max_percent.unwrap_or(100) as f32 / 100.);
            let pct_md = ((1. / prod_packing.prod.size_info.medium as f32) * 100.)
                * (prod_packing.prod.size_info.max_percent.unwrap_or(100) as f32 / 100.);

            while box_filled_pct <= 100.
                // we can allow a little bit of wiggle room.. hence u32 conversion and <=
                && box_filled_pct as u32
                    + if large_box {
                        pct_lg as u32
                    } else {
                        pct_md as u32
                    }
                    <= 100
                && prod_packing.prod.size_info.max_percent.and_then(|v| Some(v >= box_filled_pct as u32)).unwrap_or(true)
            {
                tracing::debug!(
                    "packing box with {:#?}\nthere are {} remaining\ntakes up {}% of a large box\ntakes up {}% of a medium box",
                    prod_packing.prod.info.name,
                    prod_packing.amt,
                    pct_lg,
                    pct_md
                );
                total_pct_lg -= pct_lg;
                total_pct_md -= pct_md;
                box_filled_pct += {
                    if large_box {
                        pct_lg
                    } else {
                        pct_md
                    }
                };
                tracing::debug!(
                    "current {} box filled {}%",
                    if large_box { "LARGE" } else { "MEDIUM" },
                    box_filled_pct
                );
                prod_packing.amt -= 1;
                if prod_packing.amt == 0 {
                    prods_to_remove.push(id.to_owned());
                    break;
                }
            }
        }
        tracing::debug!(
            "total lg pct remain: {}%\ntotal md pct remain: {}%",
            total_pct_lg,
            total_pct_md
        );
        if large_box {
            lg += 1;
        } else {
            md += 1;
        }
        prods_to_remove.drain(..).into_iter().for_each(|id| {
            let removed = prods.remove(&id);
            tracing::debug!("removed: {:#?}", removed.and_then(|p| p.prod.info.name))
        });
    }

    ShippingInfo::Box { md, lg }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::LazyLock};

    use crate::TRACING;

    use super::get_shipping_info;

    #[test]
    fn test_shipping_info() {
        dotenv::dotenv().ok();
        LazyLock::force(&TRACING);
        let hoodie_id = "prod_RoGl6KP2pm8O4a";
        let shirt_id = "prod_Rn8uJGQ7eaGYBA";
        let hat_id = "prod_RoI1rlvwNGH1zn";

        let mut items = std::collections::HashMap::new();
        items.insert(stripe::ProductId::from_str(hoodie_id).unwrap(), 5);
        items.insert(stripe::ProductId::from_str(shirt_id).unwrap(), 20);
        let expected = super::ShippingInfo::Box { md: 0, lg: 3 };
        let got = super::get_shipping_info(&items);
        assert_eq!(expected, got);
        let _ = items.drain();

        items.insert(stripe::ProductId::from_str(hat_id).unwrap(), 10);
        let expected = super::ShippingInfo::Box { md: 1, lg: 1 };
        let got = super::get_shipping_info(&items);
        assert_eq!(expected, got);
        let _ = items.drain();

        items.insert(stripe::ProductId::from_str(hat_id).unwrap(), 5);
        items.insert(stripe::ProductId::from_str(shirt_id).unwrap(), 3);
        let expected = super::ShippingInfo::Box { md: 1, lg: 0 };
        let got = super::get_shipping_info(&items);
        assert_eq!(expected, got);
        let _ = items.drain();

        items.insert(stripe::ProductId::from_str(hat_id).unwrap(), 8);
        items.insert(stripe::ProductId::from_str(shirt_id).unwrap(), 5);
        let expected = super::ShippingInfo::Box { md: 0, lg: 1 };
        let got = super::get_shipping_info(&items);
        assert_eq!(expected, got);
        let _ = items.drain();

        items.insert(stripe::ProductId::from_str(hoodie_id).unwrap(), 1);
        items.insert(stripe::ProductId::from_str(shirt_id).unwrap(), 3);
        let expected = super::ShippingInfo::Box { md: 1, lg: 0 };
        let got = super::get_shipping_info(&items);
        assert_eq!(expected, got);
        let _ = items.drain();
    }
}
