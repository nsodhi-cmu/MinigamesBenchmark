//! Every game there is, by name.

use crate::game::{Game, Spec};
use crate::games;

pub struct Entry {
    pub spec: Spec,
    pub build: fn(u64) -> Box<dyn Game>,
}

/// In the order the titles read, which is the order the menu shows them in.
pub const CATALOG: &[Entry] = &[
    Entry { spec: games::arena::SPEC, build: games::arena::build },
    Entry { spec: games::balance::SPEC, build: games::balance::build },
    Entry { spec: games::balloon::SPEC, build: games::balloon::build },
    Entry { spec: games::beam::SPEC, build: games::beam::build },
    Entry { spec: games::cipher::SPEC, build: games::cipher::build },
    Entry { spec: games::circuit::SPEC, build: games::circuit::build },
    Entry { spec: games::cleaver::SPEC, build: games::cleaver::build },
    Entry { spec: games::circles::SPEC, build: games::circles::build },
    Entry { spec: games::clocked::SPEC, build: games::clocked::build },
    Entry { spec: games::census::SPEC, build: games::census::build },
    Entry { spec: games::conveyor::SPEC, build: games::conveyor::build },
    Entry { spec: games::curl::SPEC, build: games::curl::build },
    Entry { spec: games::duo::SPEC, build: games::duo::build },
    Entry { spec: games::dash::SPEC, build: games::dash::build },
    Entry { spec: games::eye::SPEC, build: games::eye::build },
    Entry { spec: games::rocks::SPEC, build: games::rocks::build },
    Entry { spec: games::flip::SPEC, build: games::flip::build },
    Entry { spec: games::fruits::SPEC, build: games::fruits::build },
    Entry { spec: games::galaxy::SPEC, build: games::galaxy::build },
    Entry { spec: games::ghost::SPEC, build: games::ghost::build },
    Entry { spec: games::gravity::SPEC, build: games::gravity::build },
    Entry { spec: games::glyph::SPEC, build: games::glyph::build },
    Entry { spec: games::guards::SPEC, build: games::guards::build },
    Entry { spec: games::hex::SPEC, build: games::hex::build },
    Entry { spec: games::fly::SPEC, build: games::fly::build },
    Entry { spec: games::hue::SPEC, build: games::hue::build },
    Entry { spec: games::typer::SPEC, build: games::typer::build },
    Entry { spec: games::isles::SPEC, build: games::isles::build },
    Entry { spec: games::quotients::SPEC, build: games::quotients::build },
    Entry { spec: games::jam::SPEC, build: games::jam::build },
    Entry { spec: games::jump::SPEC, build: games::jump::build },
    Entry { spec: games::jungle::SPEC, build: games::jungle::build },
    Entry { spec: games::keys::SPEC, build: games::keys::build },
    Entry { spec: games::lights::SPEC, build: games::lights::build },
    Entry { spec: games::latin::SPEC, build: games::latin::build },
    Entry { spec: games::maze2::SPEC, build: games::maze2::build },
    Entry { spec: games::maze4::SPEC, build: games::maze4::build },
    Entry { spec: games::mimic::SPEC, build: games::mimic::build },
    Entry { spec: games::mines::SPEC, build: games::mines::build },
    Entry { spec: games::maze3::SPEC, build: games::maze3::build },
    Entry { spec: games::maze::SPEC, build: games::maze::build },
    Entry { spec: games::paddle::SPEC, build: games::paddle::build },
    Entry { spec: games::paint::SPEC, build: games::paint::build },
    Entry { spec: games::pairs::SPEC, build: games::pairs::build },
    Entry { spec: games::pack::SPEC, build: games::pack::build },
    Entry { spec: games::pallets::SPEC, build: games::pallets::build },
    Entry { spec: games::pyro::SPEC, build: games::pyro::build },
    Entry { spec: games::quiz::SPEC, build: games::quiz::build },
    Entry { spec: games::race::SPEC, build: games::race::build },
    Entry { spec: games::reflexes::SPEC, build: games::reflexes::build },
    Entry { spec: games::rings::SPEC, build: games::rings::build },
    Entry { spec: games::shuttle::SPEC, build: games::shuttle::build },
    Entry { spec: games::ski::SPEC, build: games::ski::build },
    Entry { spec: games::sword::SPEC, build: games::sword::build },
    Entry { spec: games::slide::SPEC, build: games::slide::build },
    Entry { spec: games::sling::SPEC, build: games::sling::build },
    Entry { spec: games::slots::SPEC, build: games::slots::build },
    Entry { spec: games::setdown::SPEC, build: games::setdown::build },
    Entry { spec: games::spire::SPEC, build: games::spire::build },
    Entry { spec: games::stones::SPEC, build: games::stones::build },
    Entry { spec: games::serpent::SPEC, build: games::serpent::build },
    Entry { spec: games::targets::SPEC, build: games::targets::build },
    Entry { spec: games::tiles::SPEC, build: games::tiles::build },
    Entry { spec: games::traffic::SPEC, build: games::traffic::build },
    Entry { spec: games::tunnel::SPEC, build: games::tunnel::build },
    Entry { spec: games::twins::SPEC, build: games::twins::build },
    Entry { spec: games::caverns::SPEC, build: games::caverns::build },
    Entry { spec: games::volley::SPEC, build: games::volley::build },
    Entry { spec: games::wire::SPEC, build: games::wire::build },
    Entry { spec: games::search::SPEC, build: games::search::build },
];

pub fn find(name: &str) -> Option<&'static Entry> {
    CATALOG.iter().find(|entry| entry.spec.name == name)
}

pub fn names() -> impl Iterator<Item = &'static str> {
    CATALOG.iter().map(|entry| entry.spec.name)
}

#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;
