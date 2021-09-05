//! What we are sending to the templates when rendering them
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use serde_derive::Serialize;
use tera::{Map, Value};

use crate::content::{Page, Section};
use crate::library::Library;
use rendering::Heading;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TranslatedContent<'a> {
    lang: &'a str,
    permalink: &'a str,
    title: &'a Option<String>,
    /// The path to the markdown file; useful for retrieving the full page through
    /// the `get_page` function.
    path: &'a Path,
}

impl<'a> TranslatedContent<'a> {
    // copypaste eh, not worth creating an enum imo
    pub fn find_all_sections(section: &'a Section, library: &'a Library) -> Vec<Self> {
        let mut translations = vec![];

        #[allow(clippy::or_fun_call)]
        for key in library
            .translations
            .get(&section.file.canonical)
            .or(Some(&HashSet::new()))
            .unwrap()
            .iter()
        {
            let other = library.get_section_by_key(*key);
            translations.push(TranslatedContent {
                lang: &other.lang,
                permalink: &other.permalink,
                title: &other.meta.title,
                path: &other.file.path,
            });
        }

        translations
    }

    pub fn find_all_pages(page: &'a Page, library: &'a Library) -> Vec<Self> {
        let mut translations = vec![];

        #[allow(clippy::or_fun_call)]
        for key in
            library.translations.get(&page.file.canonical).or(Some(&HashSet::new())).unwrap().iter()
        {
            let other = library.get_page_by_key(*key);
            translations.push(TranslatedContent {
                lang: &other.lang,
                permalink: &other.permalink,
                title: &other.meta.title,
                path: &other.file.path,
            });
        }

        translations
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingPage<'a> {
    relative_path: &'a str,
    content: &'a str,
    permalink: &'a str,
    slug: &'a str,
    ancestors: Vec<&'a str>,
    title: &'a Option<String>,
    description: &'a Option<String>,
    updated: &'a Option<String>,
    date: &'a Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    taxonomies: &'a HashMap<String, Vec<String>>,
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    summary: &'a Option<String>,
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    assets: &'a [String],
    draft: bool,
    lang: &'a str,
    lighter: Option<Box<SerializingPage<'a>>>,
    heavier: Option<Box<SerializingPage<'a>>>,
    earlier_updated: Option<Box<SerializingPage<'a>>>,
    later_updated: Option<Box<SerializingPage<'a>>>,
    earlier: Option<Box<SerializingPage<'a>>>,
    later: Option<Box<SerializingPage<'a>>>,
    title_prev: Option<Box<SerializingPage<'a>>>,
    title_next: Option<Box<SerializingPage<'a>>>,
    translations: Vec<TranslatedContent<'a>>,
}

impl<'a> SerializingPage<'a> {
    /// Grabs all the data from a page, including sibling pages
    pub fn from_page(page: &'a Page, library: &'a Library) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let pages = library.pages();
        let lighter = page
            .lighter
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let heavier = page
            .heavier
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let earlier_updated = page
            .earlier_updated
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let later_updated = page
            .later_updated
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let earlier = page
            .earlier
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let later = page
            .later
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let title_prev = page
            .title_prev
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let title_next = page
            .title_next
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let ancestors = page
            .ancestors
            .iter()
            .map(|k| library.get_section_by_key(*k).file.relative.as_str())
            .collect();

        let translations = TranslatedContent::find_all_pages(page, library);

        SerializingPage {
            relative_path: &page.file.relative,
            ancestors,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            updated: &page.meta.updated,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            toc: &page.toc,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.meta.draft,
            lang: &page.lang,
            lighter,
            heavier,
            earlier_updated,
            later_updated,
            earlier,
            later,
            title_prev,
            title_next,
            translations,
        }
    }

    /// currently only used in testing
    pub fn get_title(&'a self) -> &'a Option<String> {
        self.title
    }

    /// Same as from_page but does not fill sibling pages
    pub fn from_page_basic(page: &'a Page, library: Option<&'a Library>) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let ancestors = if let Some(lib) = library {
            page.ancestors
                .iter()
                .map(|k| lib.get_section_by_key(*k).file.relative.as_str())
                .collect()
        } else {
            vec![]
        };

        let translations = if let Some(lib) = library {
            TranslatedContent::find_all_pages(page, lib)
        } else {
            vec![]
        };

        SerializingPage {
            relative_path: &page.file.relative,
            ancestors,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            updated: &page.meta.updated,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            toc: &page.toc,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.meta.draft,
            lang: &page.lang,
            lighter: None,
            heavier: None,
            earlier_updated: None,
            later_updated: None,
            earlier: None,
            later: None,
            title_prev: None,
            title_next: None,
            translations,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingSection<'a> {
    relative_path: &'a str,
    content: &'a str,
    permalink: &'a str,
    draft: bool,
    ancestors: Vec<&'a str>,
    title: &'a Option<String>,
    description: &'a Option<String>,
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    lang: &'a str,
    assets: &'a [String],
    pages: Vec<SerializingPage<'a>>,
    subsections: Vec<&'a str>,
    translations: Vec<TranslatedContent<'a>>,
    includers: Vec<&'a str>,
}

impl<'a> SerializingSection<'a> {
    pub fn from_section(section: &'a Section, library: &'a Library) -> Self {
        let mut pages = Vec::with_capacity(section.pages.len());
        let mut subsections = Vec::with_capacity(section.subsections.len());
        let mut includers = Vec::with_capacity(section.includers.len());

        for k in &section.pages {
            pages.push(library.get_page_by_key(*k).to_serialized_basic(library));
        }

        for k in &section.subsections {
            subsections.push(library.get_section_path_by_key(*k));
        }

        for k in &section.includers {
            includers.push(library.get_section_path_by_key(*k));
        }

        let ancestors = section
            .ancestors
            .iter()
            .map(|k| library.get_section_by_key(*k).file.relative.as_str())
            .collect();
        let translations = TranslatedContent::find_all_sections(section, library);

        SerializingSection {
            relative_path: &section.file.relative,
            ancestors,
            draft: section.meta.draft,
            content: &section.content,
            permalink: &section.permalink,
            title: &section.meta.title,
            description: &section.meta.description,
            extra: &section.meta.extra,
            path: &section.path,
            components: &section.components,
            toc: &section.toc,
            word_count: section.word_count,
            reading_time: section.reading_time,
            assets: &section.serialized_assets,
            lang: &section.lang,
            pages,
            subsections,
            translations,
            includers,
        }
    }

    /// Same as from_section but doesn't fetch pages
    pub fn from_section_basic(section: &'a Section, library: Option<&'a Library>) -> Self {
        let mut ancestors = vec![];
        let mut translations = vec![];
        let mut subsections = vec![];
        let mut includers = vec![];
        if let Some(lib) = library {
            ancestors = section
                .ancestors
                .iter()
                .map(|k| lib.get_section_by_key(*k).file.relative.as_str())
                .collect();
            translations = TranslatedContent::find_all_sections(section, lib);
            subsections =
                section.subsections.iter().map(|k| lib.get_section_path_by_key(*k)).collect();
            includers = section.includers.iter().map(|k| lib.get_section_path_by_key(*k)).collect();
        }

        SerializingSection {
            relative_path: &section.file.relative,
            ancestors,
            draft: section.meta.draft,
            content: &section.content,
            permalink: &section.permalink,
            title: &section.meta.title,
            description: &section.meta.description,
            extra: &section.meta.extra,
            path: &section.path,
            components: &section.components,
            toc: &section.toc,
            word_count: section.word_count,
            reading_time: section.reading_time,
            assets: &section.serialized_assets,
            lang: &section.lang,
            pages: vec![],
            subsections,
            translations,
            includers,
        }
    }
}
