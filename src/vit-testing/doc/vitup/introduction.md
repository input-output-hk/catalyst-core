# vitup

Vitup is a cli project which is capable to bootstrap catalyst backend which can be exercised by various tools. 
Initial purpose is to provide simple localhost backend for catalyst voting app. 

## build

before building vitup all dependencies need to be installed.
- valgrind
- jormungandr
- vit-servicing-station

then in order to build vitup in main project folder run:
`cargo build`

and install:

`cargo install --path vitup`

## quick start

The simplest configuration is available by using command:

`vitup start quick`

default endpoint will be exposed at `0.0.0.0:80` all data dumped to `.\catalyst`