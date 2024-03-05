
## Scripting
- Could have a component type like OnUpdate, OnStart, OnEnd, OnEvent, etc. that would be called by the engine when the event occurs.
- This would be easier for some users more familiar with non-ecs engines like Unity.

## ECS serialization
- This was a priority, but I'm not sure if it's still necessary. I think it would be better to focus on the engine.
  The idea was to have a way to serialize/deserialize ECS data to/from a file. This would be useful for saving/loading game states, for example. But on second thought, the user handling save/load states could just use the ECS API to do that.
  If I tried to write a catch-all serialization system, it would be very complex and would probably not be as efficient as the user doing it themselves. Rust ABI isn't stable, so I can't serialize data properly either.

## ECS heirarchy
- I think it would be useful to have a way to organize entities in a hierarchy. This would be useful for things like parenting, but also for organizing the scene in the editor. I'm not sure how this would work with the ECS, though. How it'll
  be done is to be determined.

## ultraviolet::transform
- Look at this and how it can be used to optimise transform calculations.