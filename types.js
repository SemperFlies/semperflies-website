// Taken from the async-stripe crate
/**
 * @typedef {Object} Product
 * @property {string} id - Unique identifier for the object.
 * @property {boolean | undefined} [active] - Whether the product is currently available for purchase.
 * @property {number | undefined} [created] - Time at which the object was created, measured in seconds since the Unix epoch.
 * @property {Price | undefined} [default_price] - The ID of the Price object that is the default price for this product.
 * @property {boolean} deleted - Whether the product is deleted.
 * @property {string | undefined} [description] - The product's description, meant to be displayable to the customer.
 * @property {Array<ProductFeature> | undefined} [features] - A list of up to 15 features for this product.
 * @property {Array<string> | undefined} [images] - A list of up to 8 URLs of images for this product.
 * @property {boolean | undefined} [livemode] - Has the value true if the object exists in live mode or the value false if the object exists in test mode.
 * @property {Metadata | undefined} [metadata] - Set of key-value pairs that you can attach to an object.
 * @property {string | undefined} [name] - The product's name, meant to be displayable to the customer.
 * @property {PackageDimensions | undefined} [package_dimensions] - The dimensions of this product for shipping purposes.
 * @property {boolean | undefined} [shippable] - Whether this product is shipped (i.e., physical goods).
 * @property {string | undefined} [statement_descriptor] - Extra information about a product that will appear on your customer’s credit card statement.
 * @property {TaxCode | undefined} [tax_code] - A tax code ID.
 * @property {ProductType | undefined} [type_] - The type of the product: either 'good' or 'service'.
 * @property {string | undefined} [unit_label] - A label that represents units of this product.
 * @property {number | undefined} [updated] - Time at which the object was last updated, measured in seconds since the Unix epoch.
 * @property {string | undefined} [url] - A URL of a publicly-accessible webpage for this product.
 */


/**
 * @typedef {Object} Price
 * @property {string} id - The unique identifier of the price.
 * @property {number} amount - The price amount in the smallest currency unit (e.g., cents).
 * @property {string} currency - The currency of the price.
 * @property {boolean} [active] - Whether the price can be used for new purchases.
 * @property {string} [billing_scheme] - Describes how to compute the price per period. Either `per_unit` or `tiered`.
 * @property {number} [created] - Time at which the object was created (Unix timestamp).
 * @property {string} [currency_options] - Prices defined in each available currency option.
 * @property {Object} [custom_unit_amount] - Configuration for the amount to be adjusted by the customer during Checkout Sessions and Payment Links.
 * @property {boolean} [deleted] - Always true for a deleted object.
 * @property {boolean} [livemode] - Indicates if the object exists in live mode (true) or test mode (false).
 * @property {string} [lookup_key] - A lookup key used to retrieve prices dynamically from a static string.
 * @property {Object} [metadata] - A set of key-value pairs that you can attach to an object.
 * @property {string} [nickname] - A brief description of the price, hidden from customers.
 * @property {string} [product] - The ID of the product this price is associated with.
 * @property {Object} [recurring] - The recurring components of a price, such as `interval` and `usage_type`.
 * @property {string} [tax_behavior] - Specifies whether the price is considered inclusive of taxes, exclusive of taxes, or unspecified.
 * @property {Array} [tiers] - Each element represents a pricing tier (only required if `billing_scheme` is `tiered`).
 * @property {string} [tiers_mode] - Defines if the tiering price should be `graduated` or `volume` based.
 * @property {Object} [transform_quantity] - Apply a transformation to the reported usage or set quantity before computing the amount billed.
 * @property {string} [type_] - One of `one_time` or `recurring` depending on whether the price is for a one-time purchase or a recurring (subscription) purchase.
 * @property {number} [unit_amount] - The unit amount in cents (or local equivalent) to be charged, represented as a whole integer.
 * @property {string} [unit_amount_decimal] - The unit amount in cents (or local equivalent) to be charged, represented as a decimal string with at most 12 decimal places.
 */


/**
 * @typedef {Object} ProductFeature
 * @property {string} feature_name - The name of the product feature.
 * @property {string} feature_description - A description of the feature.
 */

/**
 * @typedef {Object} Metadata
 * @property {Object.<string, string>} - Key-value pairs to store additional structured information about the object.
 */

/**
 * @typedef {Object} PackageDimensions
 * @property {number} length - The length of the package in centimeters.
 * @property {number} width - The width of the package in centimeters.
 * @property {number} height - The height of the package in centimeters.
 * @property {number} weight - The weight of the package in grams.
 */

/**
 * @typedef {Object} TaxCode
 * @property {string} id - The unique identifier of the tax code.
 */

/**
 * @typedef {('good' | 'service')} ProductType
 * - 'good' is eligible for use with Orders and SKUs.
 * - 'service' is eligible for use with Subscriptions and Plans.
 */

/**
 * @typedef {Object} ShoppingCart
 * @property {Object<string, ProductInCart>} items 
 */

 /**
 * @typedef {Object} ProductInCart
 * @property {number}  quantity
 * @property {number} price - the price of the item in cents
 * @property {string} name 
 * @property {Array<string> | undefined} [images] - A list of up to 8 URLs of images for this product.
 * @property {string | undefined} [description] - The product's description, meant to be displayable to the customer.
 */

 
/**
 * @typedef {Object} Address
 * @property {string} Line1 - The first line of the address.
 * @property {string} Line2 - The second line of the address (optional).
 * @property {string} ZipCode - The postal code for the address.
 * @property {string} State - The state abbreviation (e.g., "CA", "NY").
 * @property {string} City - The city for the address.
 */

/**
 * @typedef {Object} customerCredentials
 * @property {string} name - The customer's name.
 * @property {string} email - The customer's email address.
 * @property {Address} address - The customer's address.
 */
