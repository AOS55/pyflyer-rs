#[macro_export]
macro_rules! extract_or_default {
    ($dict:expr, $key:expr, $default:expr) => {
        $dict
            .get_item($key)
            .ok()
            .and_then(|bound| bound.extract::<f64>().ok())
            .unwrap_or($default)
    };
}
