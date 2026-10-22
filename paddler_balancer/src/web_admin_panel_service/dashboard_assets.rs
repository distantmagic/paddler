use std::collections::BTreeSet;

use esbuild_metafile::asset::Asset;
use esbuild_metafile::esbuild_metafile::EsbuildMetafile;
use esbuild_metafile::input_lookup::InputLookup;
use esbuild_metafile::input_properties::InputProperties;
use esbuild_metafile::path_renderer::PathRenderer;
use esbuild_metafile::preloadable_asset::PreloadableAsset;

const DASHBOARD_INPUT_PATHS: [&str; 2] = [
    "resources/css/page-admin-panel.css",
    "resources/ts/controller_dashboard.tsx",
];

pub struct DashboardAssets {
    pub html: String,
}

impl DashboardAssets {
    #[must_use]
    pub fn of(esbuild_metafile: &EsbuildMetafile) -> Option<Self> {
        let path_renderer = PathRenderer {};
        let mut rendered_includes = BTreeSet::new();
        let mut rendered_preloads = BTreeSet::new();

        for input_path in DASHBOARD_INPUT_PATHS {
            let InputLookup::Found(InputProperties {
                outputs, preloads, ..
            }) = esbuild_metafile.input(input_path)
            else {
                return None;
            };

            rendered_preloads.extend(preloads.into_iter().map(|preload_path| {
                PreloadableAsset::from_path(preload_path).render(&path_renderer)
            }));

            for output_path in outputs {
                match Asset::from_path(output_path) {
                    Asset::Unknown(_) => {}
                    includable_asset => {
                        rendered_includes.insert(includable_asset.render(&path_renderer));
                    }
                }
            }
        }

        let mut html = String::new();

        for rendered_tag in rendered_preloads.into_iter().chain(rendered_includes) {
            html.push_str(&rendered_tag);
            html.push('\n');
        }

        Some(Self { html })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use esbuild_metafile::esbuild_metafile::EsbuildMetafile;

    use super::DashboardAssets;

    const DASHBOARD_METAFILE: &str = r#"{
        "outputs": {
            "static/controller_dashboard.js": {
                "imports": [{ "path": "static/icon.svg" }],
                "cssBundle": "static/controller_dashboard.css",
                "entryPoint": "resources/ts/controller_dashboard.tsx"
            },
            "static/controller_dashboard.css": {
                "imports": [{ "path": "static/icon.svg" }],
                "cssBundle": null,
                "entryPoint": null
            },
            "static/page-admin-panel.css": {
                "imports": [{ "path": "static/font.ttf" }],
                "cssBundle": null,
                "entryPoint": "resources/css/page-admin-panel.css"
            },
            "static/icon.svg": {
                "imports": [],
                "cssBundle": null,
                "entryPoint": null,
                "inputs": { "resources/icons/icon.svg": {} }
            },
            "static/font.ttf": {
                "imports": [],
                "cssBundle": null,
                "entryPoint": null,
                "inputs": { "resources/fonts/font.ttf": {} }
            }
        }
    }"#;

    #[test]
    fn renders_every_preload_before_the_includes_of_both_dashboard_inputs() {
        let esbuild_metafile = EsbuildMetafile::from_str(DASHBOARD_METAFILE).unwrap();

        assert_eq!(
            DashboardAssets::of(&esbuild_metafile).unwrap().html,
            concat!(
                "<link rel=\"preload\" href=\"/static/font.ttf\" as=\"font\" crossorigin>\n",
                "<link rel=\"preload\" href=\"/static/icon.svg\" as=\"image\">\n",
                "<link rel=\"stylesheet\" href=\"/static/controller_dashboard.css\">\n",
                "<link rel=\"stylesheet\" href=\"/static/page-admin-panel.css\">\n",
                "<script async src=\"/static/controller_dashboard.js\" type=\"module\"></script>\n",
            )
        );
    }

    #[test]
    fn a_metafile_without_a_dashboard_input_describes_no_dashboard_assets() {
        let esbuild_metafile = EsbuildMetafile::from_str(r#"{ "outputs": {} }"#).unwrap();

        assert!(DashboardAssets::of(&esbuild_metafile).is_none());
    }
}
