use std::collections::HashMap;

use super::products::{get_cached_products, Product};

// THESE WILL NEED TO BE UPDATED WHEN THE STRIPE ACCOUNT IS MIGRATED
pub const LARGE_BOX_ID: &str = "prod_RvMM3Uewc20w9Q";
pub const MD_BOX_ID: &str = "prod_RvMLcnHH851e2b";

/// How many boxes of each size are required
#[derive(Debug, PartialEq)]
pub struct ShippingInfo {
    pub md: usize,
    pub lg: usize,
}

#[derive(Debug)]
struct ProductPacking {
    prod: Product,
    amt: usize,
    percent_md: f32,
    percent_lg: f32,
}

#[tracing::instrument(name = "get shipping info")]
fn get_shipping_info(items: HashMap<stripe::ProductId, usize>) -> ShippingInfo {
    let cached = get_cached_products();
    // let total_amt_items = items.iter().fold(0, |acc, (_, v)| acc + v);
    let mut total_pct_md = 0.;
    let mut total_pct_lg = 0.;
    let mut prods = HashMap::new();
    for (id, v) in cached.into_iter() {
        if let (Some(p), Some(amt)) = (Product::try_from(v).ok(), items.get(&id)) {
            let percent_md = (*amt as f32 / p.size_info.medium as f32) * 100.;
            let percent_lg = (*amt as f32 / p.size_info.large as f32) * 100.;
            tracing::warn!(
                "product: {:#?}\namt: {amt}\npct medium: {percent_md}\n pct large: {percent_lg}\n",
                p.info.name
            );
            total_pct_md += percent_md;
            total_pct_lg += percent_lg;
            let packing = ProductPacking {
                prod: p,
                amt: amt.to_owned(),
                percent_md,
                percent_lg,
            };
            prods.insert(id, packing);
        }
    }
    tracing::warn!("total pct medium: {total_pct_md}\ntotal pct large: {total_pct_lg}\n");
    if total_pct_md <= 100. {
        return ShippingInfo { md: 1, lg: 0 };
    } else if total_pct_lg <= 100. {
        return ShippingInfo { md: 0, lg: 1 };
    }

    let mut md = 0;
    let mut lg = 0;
    let mut prods_to_remove = vec![];
    while !prods.is_empty() {
        let mut box_filled_pct: f32 = 0.;
        let large_box = total_pct_lg >= 100. && total_pct_md >= 100.;
        if large_box {
            tracing::warn!("FILLING LARGE");
        } else {
            tracing::warn!("FILLING MEDIUM");
        }

        for (id, prod_packing) in prods.iter_mut() {
            let pct_lg = (1. / prod_packing.prod.size_info.large as f32) * 100.;
            let pct_md = (1. / prod_packing.prod.size_info.medium as f32) * 100.;
            while box_filled_pct <= 100.
                // we can allow a little bit of wiggle room.. hence u32 conversion
                && box_filled_pct as u32
                    + if large_box {
                        pct_lg as u32
                    } else {
                        pct_md as u32
                    }
                    <= 100
            {
                if prod_packing.amt == 0 {
                    prods_to_remove.push(id.to_owned());
                    break;
                }
                tracing::warn!(
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
                tracing::warn!(
                    "current {} box filled {}%",
                    if large_box { "LARGE" } else { "MEDIUM" },
                    box_filled_pct
                );
                prod_packing.amt -= 1;
            }
        }
        tracing::warn!(
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
            tracing::warn!("removed: {:#?}", removed.and_then(|p| p.prod.info.name))
        });
    }

    ShippingInfo { md, lg }
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
        // dont yet have accurate hat size info
        // let hat_id = "prod_RoI1rlvwNGH1zn";
        let mut items = std::collections::HashMap::new();
        items.insert(stripe::ProductId::from_str(hoodie_id).unwrap(), 5);
        items.insert(stripe::ProductId::from_str(shirt_id).unwrap(), 20);
        let expected = super::ShippingInfo { md: 2, lg: 2 };
        let got = super::get_shipping_info(items);
        assert_eq!(expected, got);
    }
}
