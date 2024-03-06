use web_sys::HtmlImageElement;

pub(crate) struct Texture {
    pub id: i32,
    pub image: HtmlImageElement,
}
impl Texture {
    pub fn new(id: i32, image: HtmlImageElement) -> Self {
        Self {
            id: id,
            image: image,
        }
    }
    pub fn from_url(id: i32, url: &str) -> Self {
        let image = HtmlImageElement::new().expect("Could not create image element.");
        image.set_src(url);
        Self {
            id: id,
            image: image,
        }
    }
}
