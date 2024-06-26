#![allow(clippy::expect_fun_call)]
use std::borrow::Cow;
use std::path::PathBuf;

use sss_code::config::get_config;
use sss_code::ImageCode;
use sss_code::{list_themes, load_theme, theme_from_vim};
use sss_lib::generate_image;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

const DEFAULT_SYNTAXSET: &[u8] = include_bytes!("../../../assets/syntaxes.bin");
const DEFAULT_THEMESET: &[u8] = include_bytes!("../../../assets/themes.bin");

fn main() {
    let (config, mut g_config) = get_config();

    let cache_path = directories::BaseDirs::new()
        .unwrap()
        .cache_dir()
        .join("sss");

    let mut ss: SyntaxSet =
        if let Ok(ss) = syntect::dumps::from_dump_file(cache_path.join("syntaxes.bin")) {
            ss
        } else {
            syntect::dumps::from_binary(DEFAULT_SYNTAXSET)
        };
    let mut themes: ThemeSet =
        if let Ok(ts) = syntect::dumps::from_dump_file(cache_path.join("themes.bin")) {
            ts
        } else {
            syntect::dumps::from_binary(DEFAULT_THEMESET)
        };

    if let Some(dir) = &config.extra_syntaxes {
        let mut builder = ss.into_builder();
        builder
            .add_from_folder(dir, true)
            .expect("Cannot add syntax from folder");
        ss = builder.build();
        syntect::dumps::dump_to_file(&ss, cache_path.join("syntaxes.bin"))
            .expect("Cannot dump syntaxes to file");
    }

    if config.list_themes {
        list_themes(&themes);
        return;
    }

    if config.list_file_types {
        list_file_types(&ss);
        return;
    }

    // build cache of themes or syntaxes
    if let Some(from) = config.build_cache.as_ref() {
        let to = PathBuf::from(&g_config.output);

        themes
            .add_from_folder(from.join("themes"))
            .expect("Cannot add themes from current folder");
        let mut builder = ss.clone().into_builder();
        builder
            .add_from_folder(from.join("syntaxes"), true)
            .expect("Cannot add syntaxes from current folder");
        ss = builder.build();

        syntect::dumps::dump_to_file(&themes, to.join("themes.bin"))
            .expect("Cannot dump themes to file");
        syntect::dumps::dump_to_file(&ss, to.join("syntaxes.bin"))
            .expect("Cannot dump syntaxes to file");
        std::process::exit(0);
    }

    let content = config
        .content
        .clone()
        .expect("Cannot get content from args")
        .contents()
        .expect("Cannot get content to render");
    let syntax = if let Some(ext) = &config.extension {
        ss.find_syntax_by_extension(ext)
            .expect(&format!("Extension not found: {ext}"))
    } else {
        ss.find_syntax_by_first_line(content.split('\n').next().unwrap())
            .expect("Extension not found by code")
    };

    let theme = if let Some(vim_theme) = &config.vim_theme {
        Cow::Owned(theme_from_vim(vim_theme))
    } else {
        let theme = config
            .theme
            .clone()
            .unwrap_or("base16-ocean.dark".to_string());
        themes
            .themes
            .get(&theme)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(load_theme(&theme, false)))
    };

    if theme.settings.background.is_some()
        && g_config.colors.windows_background
            == sss_lib::Background::Solid(sss_lib::image::Rgba([0x42, 0x87, 0xf5, 255]))
    {
        g_config.colors.windows_background = theme
            .settings
            .background
            .map(|c| sss_lib::Background::Solid(sss_lib::image::Rgba([c.r, c.g, c.b, c.a])))
            .unwrap();
    }

    generate_image(
        g_config.clone(),
        ImageCode {
            config,
            syntax,
            theme,
            lib_config: g_config.clone(),
            syntax_set: &ss,
            content: &content,
            font: g_config.fonts,
        },
    );
}

fn list_file_types(ss: &SyntaxSet) {
    for s in ss.syntaxes() {
        println!("- {} (.{})", s.name, s.file_extensions.join(", ."));
    }
}
