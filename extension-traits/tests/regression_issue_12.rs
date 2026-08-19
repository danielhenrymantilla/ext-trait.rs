use extension_traits::extension;

#[extension(trait Trait)]
impl u32 {
    type Type = ();
}
