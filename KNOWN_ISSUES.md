## Blinn Phong example is sitll "blue tinted"
- Could be a problem with procedural meshes and normal generation
- Could be due to packing of vertex data not correctly referencing normals (ie snorm didn't convert correctly!)
- Could be wrong endianness packing of colors and uniform colors

## Can't launch games with command line args
- cargo run -- lighting, or whatever fails with an Error: No game session

## Mesh data is not being cleared correctly between games
- Load one game, it renders fine, close it.
- Open another game, meshes may be messed up.
- It doesn't happen 100% of the time, but when it does, the mesh just doesn't show. This usually happens during the Procedural Meshes example.

## Hello World example doesn't work anymore
- Nothing is rendered to the screen!
- No text, no "box"
- Likely due to a conflict in how packed vertex's work now between quad renderer and the mesh rendering

## Lighting Example text doesn't render text or UI
- The main sphere renders
- Text and light indicators don't render at all