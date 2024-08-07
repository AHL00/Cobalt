
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

## How will the material system work?
- A few types: PBR, Unlit, etc.
- Each type will have a shader and a set of parameters.
- First, just write an inefficient uber shader with all of the uniforms.
- Then, later on write a dynamic shader system that will generate the shader based on the parameters.
- But that means the shader will have to be recompiled every time a parameter is added or removed. I think this dynamic system
  is a bit overkill for now. Just write a shader for each material type and be done with it. However, it'd be useful for a
  material graph system, which is a long way off.
- So basically: here are the steps:
  1. Write a shader for each material type.
  2. Write a system that will set the uniforms for each material type.
  3. Write a system that will render each material type.

### Implementation details
- Should the material contain it's own pipeline? What are the alternatives?
- I could have a PBRPipeline, UnlitPipeline, etc. and the material would contain a reference to the pipeline.
- I would have to rework the current pipeline system to allow for this. Maybe sprite would still retain it's own pipeline, but
  the current system where our `RendererPipeline` is required to have it's own wgpu pipeline would have to be reworked.
- Does each renderable type need its own unique pipeline? Probably not, but we'll see as we go along.
- Where would the material pipelines be stored? In the `Renderer`?   

- Materials will have a few functions:
  - Set the uniforms to the shader on a given render pass, which will be called by the renderer.
  - Get the material's pipeline.

- Issue:
  - Models may or may not have UV coords. How to handle this? Separate pipeline for models with UV coords?
    That sounds like a pain in the ass. Do I somehow generate UVs?

## Material grouping
- When rendering, if we group by material, we can reduce the number of pipeline switches. This is a common optimization in
  game engines. To do this, should investigate a way to add the "render data" of each object to a list before rendering each frame.
- This list would have to contain renderables of many types, such as Mesh, Sprite, etc. This could post a challenge but I think
  it's easiest solved by using unsafe Rust. It shouldn't be a big deal as whenever rendering is happening, the renderer has sole
  control over the engine/world/main thread. 
- This system would probably be implemented in the main Renderer struct. Probably will need to change the RendererPipelines too.

## Multiple lights
- How do I handle lights
- On every frame, lights will be added to a buffer
- Do I do forward or deferred renderring

## Culling 
- Sphere or box bounding boxes?

## Renderer architecture changes
- Submit render queue on another thread
- Add all renderables to a list to perform things like culling and material grouping

## Jobs system
- Global, will be utilised by engine and users
- This will help prevent too many threads being created
- Or is it better to have a struct JobSystem. The renderer, physics and user will have different instances
- This will make it easier to stop user's jobs while rendering
- Could make it so jobs can't access engine resources, that means
  safety is guaranteed while running. If the user deletes a material
  resource while the engine is rendering, there will be issues as it is
  assumed resources live longer than the renderpass.
- Scripts can also be run in parallel, just have to give a warning that scripts are not guaranteed to be in any particular order

### Current finalised design
- Not global, global is again stupid because it will 100% increase the latency of subsytems as
  they have to wait for other systems to finish. Or is that good??? If we have separate 
  instances, they will wait for each other anyways, but I don't have to deal with
  scheduling.
- Prioritises engine internal jobs
- If I stop user jobs before rendering, waiting for the finishing of the task WILL cause issues. 
  So what is better in my opinion is restricting access to the Engine struct. Will asset_server
  be an issue? No because assets can't be dropped manually, they are dropped if their reference
  is dropped and that won't happen without the user accessing Engine.

## Save system
- Implementations of serde for every single cobalt type
- Try downcasting to dyn Serialize + Deserialize to see whether to save that component type

## Asset packing
- Check cherno video
- Asset system can search path for any .cpak files and load the file names present
- If the game tries to load a file, it will check if it's in the cpak file and load it from there

## Crate splitting
- Split the engine into multiple crates:
- cobalt_core: Contains the core engine functionality, like the ECS, renderer, etc.
- cobalt_runtime: Contains the runtime functionality, like asset loading, actual main thread stuff, etc.
- cobalt_editor: Contains the editor functionality, like the editor UI, building asset packs.