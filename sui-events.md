# `sui` event rehaul

the problem is stores kind of suck and they solve a problem i was too lazy to fix well, which is event handling as a whole

so what i'm thinking is events should be taken out of Layable as-is and made to take a mutable reference, essentially splitting `Layable` in two

## why this (or something similar) needs to be done

for higher-level sui components (like [`Game`](/game/src/game.rs)), storing everything in a single-threaded shared reference ([`Store`](/sui/sui/src/core/store.rs)) is clunky, infeasible and inefficient, and also probably not runtime-safe enough.

this change could elevate `sui` into pretty-ok status for game development, as we could simply implement `pass_events` and mutate our game variables from the input directly, without having a single `Store`.

## problems

- this shit's kinda intermingled
- RootContext takes a `&L`, although we could just make it take a `&mut L where L: Layable + pass_events`

- divs can take a whole variety of containers (see [`DivComponents`](/sui/sui/src/comp/div.rs)), this is actually the reason Layable needs to be split, otherwise there'd be no way to differentiate between `DivComponents` that are borrowed as a reference, thus only allowed for rendering, or fully owned or mutably borrowed, which we can pass events to too.

- how do we bridge the gap? what if we really do need to pass events events, but can only acquire a borrow (like in StageManager, where we plan to tick from the render function)?

> i guess i could make sui_runner always tick the root component but ehhhh that feels like a lot of unnecessary ticking \
> or do i give StageManager an external way to receive stages? \
> this is solved right now by just not using the stage system and loading all the assets and everything we'll need on startup but the more complex the game gets the more async shit we need to do so it's gonna come up sooner or later

- `DynamicLayable` ooof this is a big one

> `DynamicLayable` could be made events-only. kind of a bummer, but
>
> 1. events are only unimplemented if we're working with an immutable borrow
> 2. we can implement a dummy events for any immutable borrow, i'll leave an implementation from the first try in the comments here
> so `DynamicLayable` will work as it did before only it'll now be DynamicLayable+Events technically

<!-- 

use crate::Layable;

/// a wrapper for any Layable that makes it possible to implement Layable
/// for an immutable reference
///
/// &L has implemented Layable up until right now so this is here in case any code
/// anywhere depended on that
pub struct ImmutableLayable<'a, L: Layable>(pub &'a L);
impl<'a, L: Layable> ImmutableLayable<'a, L> {
	pub fn new(reference: &'a L) -> Self {
		Self(reference)
	}
}
impl<'a, L: Layable> Layable for ImmutableLayable<'a, L> {
	fn size(&self) -> (i32, i32) {
		self.0.size()
	}
	fn render(&self, d: &mut super::Handle, det: super::Details, scale: f32) {
		self.0.render(d, det, scale);
	}
	fn pass_event(
		&mut self,
		event: super::Event,
		_det: super::Details,
		_scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		println!(
			"dropped {event:?} passed to ImmutableLayable; passing events requires mutability"
		);
		None
	}
}

 -->

that's all the thoughts i have right now bye
