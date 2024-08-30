pub struct Gear {
    typ: GearType,
    price: u32,
    img_src: String,
}

pub enum GearType {
    Apparel(Apparel),
    Misc(Misc),
}

pub enum Apparel {
    TShirt,
    Hoodie,
    Hat,
}
