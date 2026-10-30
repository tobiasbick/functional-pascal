//! OS entropy is obtained without VM values, generator locks or task-owned allocation.

#[cfg(test)]
mod tests;
pub(in crate::execution::vm) mod wait;

#[cfg(test)]
type Provider = std::sync::Arc<dyn Fn(&mut [u8]) -> Result<(), getrandom::Error> + Send + Sync>;

#[derive(Default)]
pub(super) struct Source {
    #[cfg(test)]
    provider: std::sync::Mutex<Option<Provider>>,
}
impl Source {
    pub(super) fn fill(&self, bytes: &mut [u8]) -> Result<(), getrandom::Error> {
        #[cfg(test)]
        {
            let provider = self
                .provider
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            if let Some(provider) = provider {
                return provider(bytes);
            }
        }
        getrandom::fill(bytes)
    }
}
