//! Map-id canonicalization shared by every site that needs to render a
//! `map_id` to a human label.
//!
//! Centralised here because the substitution was duplicated inline
//! in seven different call sites (commands/server.rs:279, 350, 517,
//! 566, 593, 607; integrations/bridge.rs:12, 205; stub.rs:112;
//! launcher.rs:41) with subtle drift - the canonical version replaces
//! underscores with spaces and trims the trailing `_WP` suffix, the
//! inline copies only trimmed `_WP`.  Two functions also redefined
//! the operation verbatim (`map_display_label` x 2).  This module
//! collapses both forms.

/// Trim the `_WP` suffix and replace underscores with spaces.
///
/// Examples:
///   `TheIsland_WP` → `TheIsland`
///   `ScorchedEarth_WP` → `ScorchedEarth`
///   `The_Center_WP` → `The Center`
pub fn map_label(map_id: &str) -> String {
    map_id.trim_end_matches("_WP").replace('_', " ")
}

/// Trim only the `_WP` suffix, without converting the rest.  Used by
/// the launcher when it needs the raw key form (`TheIsland_WP`)
/// back from a label (`TheIsland`) and for tests that key off the
/// canonical-with-prefix identifier.
pub fn map_key_stem(map_id: &str) -> &str {
    map_id.trim_end_matches("_WP")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_label_trims_wp_and_replaces_underscores() {
        assert_eq!(map_label("TheIsland_WP"), "TheIsland");
        assert_eq!(map_label("The_Center_WP"), "The Center");
        assert_eq!(map_label("Ragnarok_WP"), "Ragnarok");
        assert_eq!(map_label("Aberration_WP"), "Aberration");
        assert_eq!(map_label("Extinction_WP"), "Extinction");
        assert_eq!(map_label("Club Ark Aberration_WP"), "Club Ark Aberration");
        assert_eq!(map_label("ScorchedEarth_WP"), "ScorchedEarth");
        assert_eq!(map_label("Genesis_Part_2_WP"), "Genesis Part 2");
        assert_eq!(map_label("Some_Map"), "Some Map");
        assert_eq!(map_label("LoneMap"), "LoneMap");
        assert_eq!(map_label(""), "");
    }

    #[test]
    fn map_key_stem_returns_input_without_wp_suffix() {
        assert_eq!(map_key_stem("TheIsland_WP"), "TheIsland");
        assert_eq!(map_key_stem("Ragnarok_WP"), "Ragnarok");
        assert_eq!(map_key_stem("Some_Map"), "Some_Map");
        assert_eq!(map_key_stem(""), "");
    }
}
