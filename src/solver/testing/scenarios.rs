use super::Scenario;

pub fn get_scenarios() -> Vec<Scenario> {
    vec![
        Scenario::new(
            "one-res-two-sources",
            vec!["browser/browser/main.ftl", "toolkit/browser/main.ftl"],
            vec![
                ("browser", vec!["en-US"], "browser"),
                ("toolkit", vec!["en-US"], "toolkit"),
            ],
            vec!["browser/main.ftl"],
            Some(vec![vec![0], vec![1]]),
        ),
        Scenario::new(
            "small",
            vec![
                "browser/branding/brand.ftl",
                "browser/menu.ftl",
                "browser/shared.ftl",
                "toolkit/branding/brand.ftl",
                "toolkit/menu.ftl",
                "toolkit/shared.ftl",
            ],
            vec![
                ("browser", vec!["en-US"], "browser"),
                ("toolkit", vec!["en-US"], "toolkit"),
            ],
            vec!["branding/brand.ftl", "menu.ftl", "shared.ftl"],
            Some(vec![
                vec![0, 0, 0],
                vec![0, 0, 1],
                vec![0, 1, 0],
                vec![0, 1, 1],
                vec![1, 0, 0],
                vec![1, 0, 1],
                vec![1, 1, 0],
                vec![1, 1, 1],
            ]),
        ),
        Scenario::new(
            "incomplete",
            vec![
                "browser/branding/brand.ftl",
                "browser/shared.ftl",
                "toolkit/menu.ftl",
                "toolkit/shared.ftl",
            ],
            vec![
                ("browser", vec!["en-US"], "browser"),
                ("toolkit", vec!["en-US"], "toolkit"),
            ],
            vec!["branding/brand.ftl", "menu.ftl", "shared.ftl"],
            Some(vec![vec![0, 1, 0], vec![0, 1, 1]]),
        ),
        Scenario::new(
            "preferences",
            vec![
                "browser/branding/brand.ftl",
                "browser/browser/branding/brandings.ftl",
                "browser/browser/branding/sync-brand.ftl",
                "browser/browser/preferences/preferences.ftl",
                "browser/browser/preferences/fonts.ftl",
                "browser/browser/featuregates/features.ftl",
                "browser/browser/preferences/addEngine.ftl",
                "browser/browser/preferences/blocklists.ftl",
                "browser/browser/preferences/clearSiteData.ftl",
                "browser/browser/preferences/colors.ftl",
                "browser/browser/preferences/connection.ftl",
                "browser/browser/preferences/languages.ftl",
                "browser/browser/preferences/permissions.ftl",
                "browser/browser/preferences/selectBookmark.ftl",
                "browser/browser/aboutDialog.ftl",
                "browser/browser/sanitize.ftl",
                "toolkit/toolkit/updates/history.ftl",
                "toolkit/security/certificates/deviceManager.ftl",
                "toolkit/security/certificates/certManager.ftl",
            ],
            vec![
                ("packaged-browser", vec!["en-US"], "browser"),
                ("packaged-toolkit", vec!["en-US"], "toolkit"),
            ],
            vec![
                "branding/brand.ftl",
                "browser/branding/brandings.ftl",
                "browser/branding/sync-brand.ftl",
                "browser/preferences/preferences.ftl",
                "browser/preferences/fonts.ftl",
                "browser/featuregates/features.ftl",
                "browser/preferences/addEngine.ftl",
                "browser/preferences/blocklists.ftl",
                "browser/preferences/clearSiteData.ftl",
                "browser/preferences/colors.ftl",
                "browser/preferences/connection.ftl",
                "browser/preferences/languages.ftl",
                "browser/preferences/permissions.ftl",
                "browser/preferences/selectBookmark.ftl",
                "browser/aboutDialog.ftl",
                "browser/sanitize.ftl",
                "toolkit/updates/history.ftl",
                "security/certificates/deviceManager.ftl",
                "security/certificates/certManager.ftl",
            ],
            Some(vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            ]]),
        ),
        Scenario::new(
            "langpack",
            vec![
                "packaged/browser/branding/brand.ftl",
                "packaged/browser/menu.ftl",
                "packaged/browser/shared.ftl",
                "packaged/toolkit/branding/brand.ftl",
                "packaged/toolkit/menu.ftl",
                "packaged/toolkit/shared.ftl",
                "langpack/browser/branding/brand.ftl",
                "langpack/browser/menu.ftl",
                "langpack/browser/shared.ftl",
                "langpack/toolkit/branding/brand.ftl",
                "langpack/toolkit/menu.ftl",
                "langpack/toolkit/shared.ftl",
            ],
            vec![
                ("packaged-browser", vec!["en-US"], "packaged/browser"),
                ("packaged-toolkit", vec!["en-US"], "packaged/toolkit"),
                ("langpack-browser", vec!["en-US"], "langpack/browser"),
                ("langpack-toolkit", vec!["en-US"], "langpack/toolkit"),
            ],
            vec!["branding/brand.ftl", "menu.ftl", "shared.ftl"],
            None,
        ),
    ]
}
