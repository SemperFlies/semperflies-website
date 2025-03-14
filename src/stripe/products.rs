use std::collections::HashMap;
use stripe::{Product as StripeProduct, ProductId};

use super::shipping::ShippingSize;

pub type CategorizedProducts = HashMap<ProductCategory, Vec<Product>>;
/// Stored in a file on the server, intermittently gotten through a cron job
pub type CachedProducts = HashMap<ProductId, StripeProduct>;
pub const MD_BOX_KEY: &str = "medium";
pub const LG_BOX_KEY: &str = "large";
pub const CATEGORY_KEY: &str = "category";
pub const SINGLE_KEY: &str = "single";
/// For products that are not merchandise
pub const IGNORE_KEY: &str = "ignore";

#[derive(Debug)]
pub struct Product {
    pub info: StripeProduct,
    pub size_info: SizeInformation,
    pub category: ProductCategory,
}

/// Only fails if the product is some non-merch product
impl TryFrom<StripeProduct> for Product {
    type Error = anyhow::Error;
    fn try_from(prod: StripeProduct) -> Result<Self, Self::Error> {
        if prod
            .metadata
            .as_ref()
            .is_some_and(|map| map.get(IGNORE_KEY).is_some())
        {
            return Err(anyhow::anyhow!("Non-merch product"));
        }
        let category = prod
            .metadata
            .as_ref()
            .and_then(|map| {
                map.get(CATEGORY_KEY).and_then(|k| {
                    ProductCategory::try_from(k.as_str())
                        .map_err(|err| tracing::error!("Error getting category: {err:#?}"))
                        .ok()
                })
            })
            .unwrap_or(ProductCategory::Misc);

        let size_info = prod
            .metadata
            .as_ref()
            .and_then(|map| {
                let medium: u32 = map
                    .get(MD_BOX_KEY)
                    .and_then(|v| {
                        str::parse::<u32>(v)
                            .map_err(|err| {
                                tracing::error!("Error parsing medium size to int: {err:#?}")
                            })
                            .ok()
                    })
                    .unwrap_or(category.size_info().medium);
                let large: u32 = map
                    .get(LG_BOX_KEY)
                    .and_then(|v| {
                        str::parse::<u32>(v)
                            .map_err(|err| {
                                tracing::error!("Error parsing large size to int: {err:#?}")
                            })
                            .ok()
                    })
                    .unwrap_or(category.size_info().large);

                let shipping_size = map
                    .get(SINGLE_KEY)
                    .and_then(|v| match v.as_ref() {
                        "small" => Some(ShippingSize::Small),
                        "medium" => Some(ShippingSize::Medium),
                        "large" => Some(ShippingSize::Large),
                        other => {
                            tracing::error!(
                                "encountered shipping size key that was unexpected: {other}"
                            );
                            None
                        }
                    })
                    .unwrap_or(category.size_info().shipping_size);

                Some(SizeInformation {
                    medium,
                    large,
                    max_percent: category.size_info().max_percent,
                    shipping_size,
                })
            })
            .unwrap_or(category.size_info());

        return Ok(Product {
            size_info,
            category,
            info: prod,
        });
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductCategory {
    Shirt,
    Hoodie,
    Hat,
    Beanie,
    Fly,
    Misc,
}
/// For information regaring how much of a given product fits in a medium/large box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeInformation {
    pub medium: u32,
    pub large: u32,
    /// Some items, such as hats, have a maximum in a given box but do not take up 100% of the box
    /// this field allows us to denote that
    pub max_percent: Option<u32>,
    pub shipping_size: ShippingSize,
}

impl AsRef<str> for ProductCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Shirt => "shirt",
            Self::Hoodie => "hoodie",
            Self::Hat => "hat",
            Self::Beanie => "beanie",
            Self::Fly => "fly",
            Self::Misc => "misc",
        }
    }
}

impl TryFrom<&str> for ProductCategory {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "shirt" => Ok(Self::Shirt),
            "hoodie" => Ok(Self::Hoodie),
            "hat" => Ok(Self::Hat),
            "beanie" => Ok(Self::Beanie),
            "fly" => Ok(Self::Fly),
            "misc" => Ok(Self::Misc),
            other => Err(anyhow::anyhow!("{other} is not a valid category key")),
        }
    }
}

impl ProductCategory {
    pub fn size_info(&self) -> SizeInformation {
        match self {
            Self::Shirt => SizeInformation {
                shipping_size: ShippingSize::Medium,
                max_percent: None,
                medium: 10,
                large: 15,
            },
            Self::Hoodie => SizeInformation {
                shipping_size: ShippingSize::Large,
                max_percent: None,
                medium: 2,
                large: 3,
            },
            Self::Hat => SizeInformation {
                shipping_size: ShippingSize::Large,
                max_percent: Some(65),
                medium: 5,
                large: 8,
            },
            Self::Beanie => SizeInformation {
                shipping_size: ShippingSize::Medium,
                max_percent: None,
                medium: 20,
                large: 30,
            },
            Self::Fly => SizeInformation {
                shipping_size: ShippingSize::Small,
                max_percent: None,
                medium: 250,
                large: 400,
            },
            Self::Misc => SizeInformation {
                shipping_size: ShippingSize::Small,
                max_percent: None,
                medium: 250,
                large: 400,
            },
        }
    }
}

pub fn products_path() -> std::path::PathBuf {
    match std::env::var("ENVIRONMENT")
        .expect("No ENVIRONMENT env variable")
        .to_lowercase()
        .as_str()
    {
        "prod" => {
            let home = std::env::var("HOME").expect("No HOME variable?");
            let pathstr = format!("{home}/semperflies_products.json");
            std::path::Path::new(&pathstr).to_owned()
        }
        _other => std::path::Path::new("./semperflies_products.json").to_owned(),
    }
}

pub fn get_categorized_products() -> CategorizedProducts {
    let mut categorized: CategorizedProducts = HashMap::new();
    categorized.insert(ProductCategory::Misc, vec![]);

    for (_id, prod) in get_cached_products().into_iter() {
        if let Some(product) = Product::try_from(prod).ok() {
            match categorized.get_mut(&product.category) {
                Some(vec) => vec.push(product),
                None => {
                    let _ = categorized.insert(product.category, vec![product]);
                }
            }
        }
    }

    categorized.iter_mut().for_each(|(_, vec)| {
        vec.sort_by(|a, b| {
            a.info
                .created
                .unwrap_or(i64::MAX)
                .cmp(&b.info.created.unwrap_or(i64::MAX))
        });
    });
    categorized
}

pub fn get_cached_products() -> CachedProducts {
    let str = std::fs::read_to_string(products_path()).expect("could not read path to string");
    let products: CachedProducts = serde_json::from_str(&str).expect("could not coerce to json");
    products
}
