use std::sync::{Arc, LazyLock};

use crate::extractors::chinaq::ChinaQExtractor;
use crate::extractors::SiteExtractor;

#[derive(Clone)]
pub struct ExtractorRegistry {
    extractors: Vec<Arc<dyn SiteExtractor>>,
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

impl ExtractorRegistry {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(ChinaQExtractor::new()));
        registry
    }

    pub fn register(&mut self, extractor: Arc<dyn SiteExtractor>) {
        self.extractors.push(extractor);
    }

    pub fn get_by_url(&self, url: &str) -> Option<Arc<dyn SiteExtractor>> {
        self.extractors.iter().find(|e| e.can_handle(url)).cloned()
    }

    pub fn get_by_source_id(&self, source_id: &str) -> Option<Arc<dyn SiteExtractor>> {
        self.extractors.iter().find(|e| e.id() == source_id).cloned()
    }

    pub fn all(&self) -> Vec<Arc<dyn SiteExtractor>> {
        self.extractors.clone()
    }
}

static GLOBAL_REGISTRY: LazyLock<ExtractorRegistry> =
    LazyLock::new(ExtractorRegistry::default_registry);

pub fn global_registry() -> &'static ExtractorRegistry {
    &GLOBAL_REGISTRY
}
