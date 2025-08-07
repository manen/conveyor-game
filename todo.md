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

- [ ] audio!

- SoundProvider component that catches SoundEvent return events and plays the audio, managing the audio files and loading and everything by itself (essentially just Textures at the root of the component tree)

- let RemoteStages output SoundEvents and you can play sound from other threads

---

- [ ] i'll probably need to put all the maps into an ugly ass broke ass shitty ass Arc<Mutex<_>> but ion think there's another option for much longer (off-thread ticking, checking to see if two buildings are connected with conveyors from the tutorial, etc)
