# todo

## tutorial

so the tutorial system's basically set up, next up:

- [x] sui wrapped text component

- [x] building place listener (with the tile_resource under it included too)

the 'story' of the tutorial:

- resources, resource gathering through buildings and tiles
- placing a miner
- placing conveyors

---

and this is about as long as we can get right now cause we're missing some crucial features:

- [x] disposal/collection/counting of resources by wiring them out of the world (or maybe into a house where the player lives? that'd add a nice touch) <!-- this next!  -->

- a block like that is super easy to make, `EBuilding` is already only `Clone`, just give it a Sender and let it send all the resources it receives

- the side of the map would be much more low-level, just checking to see if resources are trying to pass into a building outside bounds and add that to the count

- **verdict: building is easier and could be made more engaging too**

---

- [ ] new toolbar with textures and name only on hover (fits better and looks better)

- for that, we need to do something about textures since dropping textures actually drops the texture so that can't be cloned <!-- (*just put `sui::Texture`'s texture into an Arc<_>*) -->

- toolbar has no way of receiving a `&Textures` at render-time, so when creating the toolbar we could clone Textures and make it only hold an `Arc<HashMap<TextureID, Texture>>`, so it's cheap and easy to clone

---

- [ ] audio!

- SoundProvider component that catches SoundEvent return events and plays the audio, managing the audio files and loading and everything by itself (essentially just Textures at the root of the component tree)

- let RemoteStages output SoundEvents and you can play sound from other threads

---

- [ ] i'll probably need to put all the maps into an ugly ass broke ass shitty ass Arc<Mutex<_>> but ion think there's another option for much longer (off-thread ticking, checking to see if two buildings are connected with conveyors from the tutorial, etc)

---

- [x] ok so the thing is pass_events is really wasteful with DynamicLayables rn, so i should probably change the function signature to `(iter: impl Iterator, det, scale, ret_events: &mut Vec<ReturnEvent>);`

that'd be uglyyyy tho right now i think it's easy to use but it's really wasteful (DynamicLayables have to allocate coming in and allocate coming out). a stack of 3 dynamiclayables is 6 allocations PER PASS_EVENTS CALL (so per tick)
