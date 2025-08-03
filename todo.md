# todo

## tutorial

so the tutorial system's basically set up, next up:

- sui wrapped text component
- building place listener (with the tile_resource under it included too)

the 'story' of the tutorial:

- resources, resource gathering through buildings and tiles
- placing a miner
- placing conveyors

---

and this is about as long as we can get right now cause we're missing some crucial features:

- disposal/collection/counting of resources by wiring them out of the world (or maybe into a house where the player lives? that'd add a nice touch)

- i'll probably need to put all the maps into an ugly ass broke ass shitty ass Arc<Mutex<_>> but ion think there's another option for much longer (off-thread ticking, checking to see if two buildings are connected with conveyors from the tutorial, etc)
