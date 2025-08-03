# conveyor game

the core idea was to basically make a game like mindustry but more minigame-like.

## basic concept

### the timer

there's a timer, and you have to finish making a set amount of items before it counts down to 0

the timer only passes if the simulation is actively turned on, so the time isn't really limiting the player, rather the time needed for their factories to generate the expected resources

### resources, factories

have resources, have factories that turn those resources into other resources, etc etc etc

## implementation roadmap

### basic tiles

fillers: stone
ores: iron, coal

### buildings

- miner

mines the resource it's sitting on

- conveyor

moves the resource in the direction it's facing

- output cube

like a 3x3 cube that the target resource goes into
