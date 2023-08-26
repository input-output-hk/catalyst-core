# Build mdbook for `catalyst-core`

To build the mdbook:

`earthly -P build-docs`

This generates an artifact that can be used by other earthly targets, or
it can be extracted as a local artifact:

`earthly -P --artifact +build-docs/html ./generated_mdbook`

Running the command above will output the generated book into a folder called `generated_mbbook` in the local path.
