# vitup

Vitup is a cli project which is capable to bootstrap mock backend which can be excercised by various tools. Initial purpose is to provide simple localhost backend for 
catalyst voting app

# build

before building vitup all dependencies need to be installed.
- iapyx-proxy
- jormungandr
- vit-servicing-station

then in order to build vitup:
`cargo build`

and install:

`cargo install --path .`

# quick start

The simplest configuration is available by using command:

`vitup start quick`

default endpoint will be exposed at `0.0.0.0:80` all data dumped to `.\data\vit_backend`

# private vote type

`vitup start quick --private`

# modes

There are 3 modes available in vitup:
- interactive - where user can push some fragments or query status of nodes 
- endless - [Default] just simple run until stopped by user
- service - manager service published at `0.0.0.0:3030` and control stop/start/ and provide files over http

## service mode

- start: send POST to http://0.0.0.0:3030/control/start
- stop: send POST to http://0.0.0.0:3030/control/stop
- status: send GET to http://0.0.0.0:3030/status
- files: send GET to http://0.0.0.0:3030/files/<file-name>

# initials mapping file

Initials Mapping file is used to precisely control wallet initials. Example:
```
[
	{ "name":"jake", "funds":80 },
	{ "name":"darek", "funds":80 },
	{ "name":"juan", "funds":80 },
	{ "above_threshold":10 },
    { "below_threshold":10 },
    { "zero_funds": 10 }    
]
```

when starting vitup like below:

`vitup start quick --initials-mapping .\mappings.json --voting-power 8000`

will crates wallets:

- 1 wallet `jake` with 80 ADA
- 1 wallet `darek` with 80 ADA
- 1 wallet `juan` with 80 ADA
- 10 wallets with more than 8000 ADA
- 10 wallets below 8000 ADA
- 10 without any funds