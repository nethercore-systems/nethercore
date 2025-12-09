## Vertex Packing format is wrong
- Should use oct packing for vertex normals (code already exists for packing light data), currently uses pack_normal_snorm16
- UVs should use unorm16 not f16. Currently uses pack_uv_f16
- After these are updated, ensure tests correctly cover these cases
- Ensure the render pipeline is now referencing the correct values with the correct strides.
- This may fix the below Blinn Phong example

## Skinning Data is not packed
- Bone indices should be u8x4 or a u32 (whatever is least likely to cause problems)
- Bone weights should also be unorm8x4
- Update the pack_vertex_data to support this
- Skinning example needs to be updated

## Blinn Phong example is still "blue tinted"
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
- Likely due to a conflict in how packed vertex's work now between quad renderer and the mesh rendering pipeline, while text rendering was not updated to include this.

## Lighting Example text doesn't render text or UI
- The main sphere renders
- Text and light indicators don't render at all
- Probably due to above issue, ie packed vertex data in the pipeline.

## Textured Procedural Example is rendering the Default Text
- Likely a texture collision problem
- Default font texture is at index 0, first loaded texture is maybe also going to this address
- This causes the meshes to render with texture 0
- But could also just be a bug with the render pipeline, ie texture ids not being recorded/bound correctly