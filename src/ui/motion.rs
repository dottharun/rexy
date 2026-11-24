//! Panes take a moment to reach their places when a tab is laid out anew: one
//! that is split, closed or given room moves there, and is not suddenly there.
use std::collections::HashMap;

use eframe::egui::{self, Rect, emath::easing::cubic_out};

/// How long a pane takes to come to rest, in seconds. Zero: panes jump
/// straight to their places.
const TIME: f64 = 0.0;

/// Where a pane was when it was last sent somewhere, where it is going, and when
/// it set out.
#[derive(Clone, Copy)]
struct Place {
    from: Rect,
    to: Rect,
    since: f64,
}

impl Place {
    /// Where it is just now: it sets out fast, and slows as it arrives.
    fn shown(&self, now: f64) -> Rect {
        if TIME <= 0.0 {
            return self.to;
        }
        let part = ((now - self.since) / TIME).clamp(0.0, 1.0) as f32;
        self.from.lerp_towards(&self.to, cubic_out(part))
    }
}

/// Where the panes of the tab on show are.
#[derive(Default)]
pub struct Motion {
    /// The tab last laid out.
    tab: String,
    places: HashMap<String, Place>,
}

impl Motion {
    /// Where to draw each pane of a tab just now, given where each belongs: the
    /// panes by name, each with its part of the tab. `still` says nothing is to
    /// move, as while the pointer has hold of something. Nothing moves either
    /// when another tab comes on show: only a new arrangement of the same tab
    /// does.
    pub fn place(
        &mut self,
        ctx: &egui::Context,
        tab: &str,
        belong: &[(&str, Rect)],
        still: bool,
    ) -> Vec<Rect> {
        let now = ctx.input(|i| i.time);
        let still = still || self.tab != tab;
        tab.clone_into(&mut self.tab);
        let open = |pane: &String| belong.iter().any(|held| held.0 == pane);
        self.places.retain(|pane, _| open(pane));
        let mut moving = false;
        let shown = belong.iter().map(|&(pane, to)| {
            let set_out = |from: Rect| Place { from, to, since: now };
            let place = match self.places.get(pane) {
                _ if still => set_out(to),
                // On its way there already.
                Some(place) if place.to == to => *place,
                Some(place) => set_out(place.shown(now)),
                None => set_out(entering(to, &self.places, belong, now)),
            };
            moving |= place.from != to && now - place.since < TIME;
            match self.places.get_mut(pane) {
                Some(held) => *held = place,
                None => _ = self.places.insert(pane.to_owned(), place),
            }
            place.shown(now)
        });
        let shown = shown.collect();
        if moving {
            ctx.request_repaint();
        }
        shown
    }
}

/// Where a pane that is new to its tab comes from. It has the room of a pane
/// that was there before it and now gives way: it starts with none of that, at
/// the side away from that pane, and so takes the room as it is given up.
fn entering(to: Rect, before: &HashMap<String, Place>, belong: &[(&str, Rect)], now: f64) -> Rect {
    let gave = |&&(pane, _): &&(&str, Rect)| {
        let before = before.get(pane);
        before.is_some_and(|place| place.shown(now).contains(to.center()))
    };
    let Some(&(_, beside)) = belong.iter().find(gave) else {
        return to;
    };
    let away = to.center() - beside.center();
    let mut from = to;
    match (away.x.abs() > away.y.abs(), away.x > 0.0, away.y > 0.0) {
        (true, true, _) => from.min.x = to.max.x,
        (true, false, _) => from.max.x = to.min.x,
        (false, _, true) => from.min.y = to.max.y,
        (false, _, false) => from.max.y = to.min.y,
    }
    from
}

#[cfg(test)]
#[path = "../../tests/unit/ui_motion.rs"]
mod tests;
