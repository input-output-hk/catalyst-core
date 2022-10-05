# This is how to merge repos

Needs the tool: <https://github.com/newren/git-filter-repo>
This tool is crucial in keeping the history of the merged repo fully intact.
Every other method I tried orphaned the history.

Example using `jormungandr`

```sh
git clone git@github.com:input-output-hk/cardano-gov-world.git
cd cardano-gov-world
git checkout -b monorepo
cd ..

git clone git@github.com:input-output-hk/jormungandr.git
cd jormungandr
git filter-repo --to-subdirectory-filter src/jormungandr
cd ..

cd cardano-gov-world
git remote add jormungandr ../jormungandr
git fetch jormungandr
git merge --allow-unrelated-histories jormungandr/master
git remote remove jormungandr
```
