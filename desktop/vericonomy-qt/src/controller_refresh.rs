//! Avoid setting `loading` on background polls after the first successful fetch.

/// Returns true when the UI should enter a loading state for this refresh.
pub fn begin_poll_refresh(
    coin: &str,
    refresh_coin: &mut String,
    refresh_ready: &mut bool,
) -> bool {
    if refresh_coin.as_str() != coin {
        *refresh_coin = coin.to_string();
        *refresh_ready = false;
    }
    !*refresh_ready
}

pub fn mark_poll_ready(refresh_ready: &mut bool) {
    *refresh_ready = true;
}
