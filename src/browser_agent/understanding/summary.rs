//! Structured page summary generation.

use super::{actions::detect_actionable, forms::classify_form, links::classify_link,
    regions::detect_regions, ElementSummary, InputSummary};
use super::actions::ActionableElement;
use super::forms::FormPurpose;
use super::links::LinkKind;
use super::regions::RegionInfo;

#[derive(Clone, Debug)]
pub struct LinkInfo { pub href: String, pub text: String, pub kind: LinkKind }

#[derive(Clone, Debug)]
pub struct FormInfo { pub purpose: FormPurpose, pub input_count: usize }

#[derive(Clone, Debug, Default)]
pub struct PageSummary {
    pub regions: Vec<RegionInfo>,
    pub forms: Vec<FormInfo>,
    pub links: Vec<LinkInfo>,
    pub actions: Vec<ActionableElement>,
}

pub fn summarize_page(
    elements: &[ElementSummary],
    forms: &[Vec<InputSummary>],
    links: &[(String, String, String)],
) -> PageSummary {
    PageSummary {
        regions: detect_regions(elements),
        forms: forms.iter().map(|f| FormInfo {
            purpose: classify_form(f), input_count: f.len(),
        }).collect(),
        links: links.iter().map(|(href, text, ctx)| LinkInfo {
            href: href.clone(), text: text.clone(), kind: classify_link(href, text, ctx),
        }).collect(),
        actions: detect_actionable(elements),
    }
}
