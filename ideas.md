
## Scripting
- Could have a component type like OnUpdate, OnStart, OnEnd, OnEvent, etc. that would be called by the engine when the event occurs.
- This would be easier for some users more familiar with non-ecs engines like Unity.

## ECS optimizations
- Sacrifice memory for speed by storing components in a contiguous array instead of a hashmap.
- If an entity doesn't have a component, just store None in the array.
- This would make it easier to iterate over all entities with a certain component.
- This is what sparse set ECS's like specs do.