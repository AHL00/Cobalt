

// pub struct Arena
// pub struct ArenaPointer
// pub struct ArenaPointerMut

// An ArenaPointer will be handed out to the user and will contain a way to access the data in the arena.
// The mechanism for accessing the data will be a usize offset from the start of the arena.
// But if an object is removed from the arena, the offset of all objects after it will change.