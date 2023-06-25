# Audit Tooling: Under Construction <img width="45" alt="schermafbeelding 2017-09-27 om 23 08 12" src="https://user-images.githubusercontent.com/7254997/30937972-c9632d04-a3d8-11e7-87f3-c44ce2b86d24.png">

###

*We need to wrap this up in earthly for ease of use*

Start node with legacy storage i.e fundX with custom node config. 

```bash
./target/release/jormungandr --genesis-block /tmp/fund*-leader-1/artifacts/block0.bin --config=node_config.yaml
```

Curl for active vote plan to recreate IOG results.

```bash
curl http://127.0.0.1:10000/api/v0/vote/active/plans > voteplansgen.txt
```

Extract results from voteplansgen.txt and feed to fragments extraction tool to verify all of them match.

