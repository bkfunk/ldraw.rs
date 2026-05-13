//! Convenience wrapper for loading and baking parts by ID.
//!
//! [`PartLoader`] reduces the standard "load one part" recipe from ~7 lines
//! of ceremony (manual `.dat` suffix, `Arc<RwLock<PartCache>>`, per-call
//! `load_colors`, no-op `on_update` callback) down to a single call.
//!
//! ```no_run
//! use ldraw_ir::loader::PartLoader;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), ldraw::error::ResolutionError> {
//! let loader = PartLoader::from_library_root(PathBuf::from("/path/to/ldraw")).await?;
//! let part = loader.load_part("3001").await?;
//! # Ok(())
//! # }
//! ```

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use ldraw::{
    color::ColorCatalog,
    error::ResolutionError,
    library::{LibraryLoader, PartCache, resolve_dependencies_multipart},
    resolvers::local::LocalLoader,
    PartAlias,
};

use crate::part::{bake_part_from_multipart_document, Part};

/// A pre-warmed loader that caches colors and a part cache so individual
/// part loads don't have to.
///
/// Wraps any [`LibraryLoader`] together with its already-loaded
/// [`ColorCatalog`] and an internal [`PartCache`]. Use [`load_part`] to
/// go from a part ID to a baked [`Part`] in a single call.
///
/// [`load_part`]: PartLoader::load_part
pub struct PartLoader<L: LibraryLoader> {
    loader: L,
    colors: ColorCatalog,
    cache: Arc<RwLock<PartCache>>,
}

impl<L: LibraryLoader> PartLoader<L> {
    /// Constructs a `PartLoader` from an existing loader and color catalog.
    ///
    /// Use this when you already have a `ColorCatalog` (e.g. from a previous
    /// call to `loader.load_colors()`) and want to avoid re-reading
    /// `LDConfig.ldr`.
    pub fn new(loader: L, colors: ColorCatalog) -> Self {
        Self {
            loader,
            colors,
            cache: Arc::new(RwLock::new(PartCache::default())),
        }
    }

    /// Constructs a `PartLoader` by calling `load_colors()` on the
    /// underlying loader once, up front.
    pub async fn with_loaded_colors(loader: L) -> Result<Self, ResolutionError> {
        let colors = loader.load_colors().await?;
        Ok(Self::new(loader, colors))
    }

    /// Returns the cached [`ColorCatalog`].
    pub fn colors(&self) -> &ColorCatalog {
        &self.colors
    }

    /// Returns a reference to the underlying loader.
    pub fn inner(&self) -> &L {
        &self.loader
    }

    /// Loads and bakes the part identified by `part_id`.
    ///
    /// `part_id` may be supplied with or without an LDraw file extension
    /// (`"3001"` and `"3001.dat"` are equivalent; `"foo.ldr"` and `"foo.mpd"`
    /// are passed through). The internal [`PartCache`] is reused across
    /// successive calls, so loading dozens of parts amortizes primitive
    /// parsing.
    pub async fn load_part(&self, part_id: &str) -> Result<Part, ResolutionError> {
        let alias = alias_with_default_suffix(part_id);
        let (_, document) = self.loader.load_ref(alias, false, &self.colors).await?;
        let resolutions = resolve_dependencies_multipart(
            &document,
            Arc::clone(&self.cache),
            &self.colors,
            &self.loader,
            &|_, _| {},
        )
        .await;
        Ok(bake_part_from_multipart_document(
            &document, &resolutions, false,
        ))
    }
}

impl PartLoader<LocalLoader> {
    /// Convenience constructor: build a `PartLoader` backed by a
    /// filesystem-rooted LDraw library.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// PartLoader::with_loaded_colors(LocalLoader::new(Some(root), None)).await
    /// ```
    pub async fn from_library_root(root: PathBuf) -> Result<Self, ResolutionError> {
        Self::with_loaded_colors(LocalLoader::new(Some(root), None)).await
    }
}

/// If `part_id` has no LDraw extension (`.dat`, `.ldr`, `.mpd`), append
/// `.dat`. Trims surrounding whitespace.
fn alias_with_default_suffix(part_id: &str) -> PartAlias {
    let trimmed = part_id.trim();
    let lower = trimmed.to_lowercase();
    if lower.ends_with(".dat") || lower.ends_with(".ldr") || lower.ends_with(".mpd") {
        PartAlias::from(trimmed.to_string())
    } else {
        PartAlias::from(format!("{trimmed}.dat"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_with_default_suffix_appends_dat() {
        assert_eq!(
            alias_with_default_suffix("3001").original,
            "3001.dat",
        );
        assert_eq!(
            alias_with_default_suffix("  3001 ").original,
            "3001.dat",
            "whitespace should be trimmed before suffix check",
        );
    }

    #[test]
    fn alias_with_default_suffix_preserves_existing_extension() {
        for input in ["3001.dat", "model.ldr", "scene.mpd", "3001.DAT"] {
            assert_eq!(
                alias_with_default_suffix(input).original,
                input.trim(),
                "should not add .dat to {input}",
            );
        }
    }
}
