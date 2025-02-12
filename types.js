/**
@typedef {Object} ShopItemData
@property {string} id - The item's ID
@property {string} name - The name of the item.
@property {number} price - The price of the item.
@property {string[]} imgs - An array of image URLs for the item.
@property {string|null} description - The description of the item.
@property {string} type - the type the shop item is, correlated with the data field
@property {ClothingData | AccessoryData | FlyData | StickersData | MiscData} data - Data associated with the shop item, based on the type.
@returns {ShopItemData}

@typedef {Object} ClothingData (shirts and hoodies)
@property {string} size - The size of the clothing item.

@typedef {Object} AccessoryData (hats and beanies)

@typedef {Object} FlyData

@typedef {Object} StickersData
@property {number} diameter - size of the sticker (in inches)

@typedef {Object} MiscData


@typedef {Object.<string, ShopItemData>} CartData
An object where each key is an item ID and the value is a CartItem.
*/
