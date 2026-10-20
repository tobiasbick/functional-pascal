//! Pure, host-aware path presentation without filesystem or current-directory access.

use std::path::{Component, Path, PathBuf};

use super::normalize_path;

pub(super) fn relative_to(target: &str, base: &str) -> Option<String> {
    let target = normalize_path(target);
    let base = normalize_path(base);
    let target = Path::new(&target);
    let base = Path::new(&base);
    let target_prefix = prefix(target);
    let base_prefix = prefix(base);
    // Windows drive-relative and root-relative paths require ambient drive state.
    if [target, base]
        .into_iter()
        .any(|path| (path.has_root() || prefix(path).is_some()) && !path.is_absolute())
        || target.is_absolute() != base.is_absolute()
        || target_prefix != base_prefix
    {
        return None;
    }

    let target = relative_components(target);
    let base = relative_components(base);
    let shared = target.iter().zip(&base).take_while(|(a, b)| a == b).count();
    let base = &base[shared..];
    // Climbing back down an unmatched parent needs the missing directory name.
    if base.contains(&Component::ParentDir) {
        return None;
    }
    let mut result = PathBuf::new();
    for _ in base {
        result.push("..");
    }
    for component in &target[shared..] {
        result.push(component.as_os_str());
    }
    if result.as_os_str().is_empty() {
        result.push(".");
    }
    Some(result.to_string_lossy().into_owned())
}

fn prefix(path: &Path) -> Option<Component<'_>> {
    path.components()
        .next()
        .filter(|component| matches!(component, Component::Prefix(_)))
}

fn relative_components(path: &Path) -> Vec<Component<'_>> {
    path.components()
        .filter(|component| matches!(component, Component::Normal(_) | Component::ParentDir))
        .collect()
}

pub(super) fn change_extension(path: &str, extension: &str) -> Option<String> {
    if extension.starts_with('.')
        || extension.contains('\0')
        || extension.chars().any(std::path::is_separator)
    {
        return None;
    }
    let mut path = PathBuf::from(path);
    path.set_extension(extension)
        .then(|| path.to_string_lossy().into_owned())
}
