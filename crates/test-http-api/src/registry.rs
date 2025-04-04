use std::any::Any;
use std::sync::Arc;

use anymap3::Map;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct StateRegistry {
    inner: Arc<RwLock<Map<dyn Any + Send + Sync + 'static>>>,
}

impl StateRegistry {
    pub async fn manage<T: Send + Sync + 'static>(&self, value: T) {
        let mut map = self.inner.write().await;
        map.insert(Arc::new(value));
    }

    pub async fn state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let map = self.inner.read().await;
        map.get::<Arc<T>>().cloned()
    }

    pub async fn remove<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let mut map = self.inner.write().await;
        map.remove::<Arc<T>>()
    }
}
