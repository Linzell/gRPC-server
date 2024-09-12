// config.rs

lazy_static! {
    static ref CONFIG: Mutex<Arc<Configuration>> = Mutex::new(Arc::new(Default::default()));
}
