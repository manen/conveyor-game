pub async fn tutorial_tips() {
	change_stage(
		text_with_actions("welcome to tooltips!", [
			action("what are tooltips?", what_are_tooltips),
			action("next", place_extractor)
		])
	).await
}

pub async fn what_are_tooltips() {
	change_stage(
		text_with_actions("tooltips are little helper texts that help you master the game!", [
			action("got it", tutorial_tips)
		])
	).await
}

pub async fn place_extractor() {
	change_stage(
		text("to acquire resources, you'll need to use extractors.\nselect the small extractor from the toolbar, and place it on an ore")
	).await

	// get async iterator over blocks being placed, break out of the loop if the block we want got placed and display a new text
}

// ok so htf do i do this

// ezzel az a baj h ilyen smooth nem lehet mert az async process megkapja az eventet es vissza kell kuldeni a staget szoval fasza lenne ha ezt valahogy szebben is meg lehetne csinalni

pub enum TooltipPage {
	WhatAreTooltips,
	PlaceExtractor
}

pub async fn tutorial_tips(stage_tx, events_rx) {
	stage_tx.send(
		text_with_actions("welcome to tooltips!", [
			action("what are tooltips?", TooltipPage::WhatAreTooltips),
			action("next", TooltipPage::PlaceExtractor)
		])
	).await;

	let event = events_rx.recv().await.unwrap();

	match event {
		WhatAreTooltips => what_are_tooltips().await,
		PlaceExtractor => place_extractor().await,
	}
}
