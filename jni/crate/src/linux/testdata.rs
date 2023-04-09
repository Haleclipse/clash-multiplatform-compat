use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "testdata/"]
pub struct TestData;
