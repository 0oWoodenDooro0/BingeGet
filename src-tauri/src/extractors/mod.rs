pub mod chinaq;

pub trait SiteExtractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn can_handle(&self, url: &str) -> bool;
}
