use std::sync::LazyLock;

use tokio::sync::Mutex;

use crate::{config, lua, storage};

static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn prepare<'a>() -> tokio::sync::MutexGuard<'a, ()> {
    let guard = LOCK.lock().await;

    let mut conf = config::Configuration::default();
    conf.database.path = "sqlite::memory:".into();
    lua::setup(&["./test".into()]).unwrap();
    config::set(conf);
    storage::setup().await.unwrap();

    guard
}
