## Mesh data is not being cleared correctly between games
- Load one game, it renders fine, close it.
- Open another game, meshes may be messed up.
- It doesn't happen 100% of the time, but when it does, the mesh just doesn't show. This usually happens during the Procedural Meshes example.

## Hello World example doesn't work anymore
- Nothing is rendered to the screen!
- No text, no "box"
- Likely due to a conflict in how packed vertex's work now between quad renderer and the mesh rendering pipeline, while text rendering was not updated to include this.

## Lighting Example text doesn't render text or UI
- The main sphere renders
- Text and light indicators don't render at all
- Probably due to above issue, ie packed vertex data in the pipeline.
- Also applies to blinn phong example, in fact text doesnt render at all anymore

## Textured Procedural Example is rendering the Default Text
- Likely a texture collision problem
- Default font texture is at index 0, first loaded texture is maybe also going to this address
- This causes the meshes to render with texture 0
- But could also just be a bug with the render pipeline, ie texture ids not being recorded/bound correctly
- Might be better to explore a separate system for text vs textures, such as a FontId and TextureId to prevent this kind of problem in the future.