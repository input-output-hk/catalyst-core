# Jormungandr wallet SDK for JavaScript

This crate provides a wasm package with JavaScript and TypeScript bindings
for the Jormungandr wallet client library.

See the `Wallet` class for a starting point in the API documentation.

## Building the package

[wasm-pack](https://github.com/rustwasm/wasm-pack) is required.

```
wasm-pack build -d pkg
```

Use the `--target` option to select the target environment for the package. 

For a quicker, but unoptimized build:

```
wasm-pack build --dev
```

## Documentation

The API documentation can be generated from the built JavaScript bindings
with the following command:

```
jsdoc pkg -c ../../jsdoc.json -d pkg/doc -R README.md
```

## Tests

requirements node

```

```
