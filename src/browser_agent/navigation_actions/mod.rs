//! Agent action navigation for anchors and GET forms.
//!
//! The helpers in this module translate successful user-like actions into
//! deterministic page navigations without doing network I/O.

mod action;
mod anchor;
mod click;
mod click_user;
mod commit;
mod control;
mod dom;
mod encode;
mod entries;
mod form;
mod form_request;
mod point_action;
mod point_dispatch;
mod point_script;
mod point_target;
mod query;
mod select;
mod submit;
mod url;

#[cfg(test)]
mod request_tests;
#[cfg(test)]
mod tests;
