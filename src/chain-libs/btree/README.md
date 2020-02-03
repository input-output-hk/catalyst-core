# BTree disk-based implementation

# Current status

- Insert sync
- Batched inserts (async) (mostly to be able to setup benchmarks/run tests, but they should be usable)
- Search/lookup
- Delete and rebalance algorithm
- Keys are fixed size

# TODO

- [] Replace the storage backend for a mmap based one (and finish writing that abstraction, but that's in progress)
- [] Avoid heap allocations for read-only pages (this requires care when remapping/resizing an mmaped file)
- [] Define the user facing API.
- [] Refactor the delete algorithm 
- [] Implement deletes in flatfile? (compaction? garbage collection?)
- [] Refactor the insertion tests
- [] Make tests to leak files when failing (temp files probably?)
- [] Make benchmarks more stable?
- [] Replace partially or totally the arrayview submodule by a dependency (if there is one). Or maybe just find a way to remove the unsafe code.
- [] Implement a generic storage interface? (not really needed, only if we want to be able to switch from
mmap/fseek - fread/pread - pwrite/vectored io/direct-io without too much fuss later)
- [] Add builders