# Current design/plan

## Content storage

The actual data is stored in a separate append-only file. We could mmap this file, although using alternatives like `pread` and `pwrite` may be also good. The biggest issue with this design is that making 
deletes is more complex, as storage needs to be reclaimed/compacted. Maybe sparse files are an option? But I think not. 

## Index

The BTree is a multiversioned data structure, the algorithms on nodes operate on mutable data structures, but global modifications are done in a copy on write fashion. The idea is to keep multiple versions of the tree identified by the root and the pages in use. When a version has no readers, we can actually reuse some of its pages (the ones that a later transaction shadows) and so the file won't grow all the time and require garbage collection/compaction/desfragmentation.

Copy on write should be good for ssds, as it is not possible to modify data in place anyway, and shouldn't have that much of an impact for hdds, as the seek time will probably dominate the sequential write of the copied page. 

Failure recovery in this design can be done simply because no used data is overwritten, the idea right now do a file swap in order to make the change of versions atomic. The root pointer and the list of free pages are stored in a separate file, so we can rewrite that file after the node modifications are on disk, and then reuse pages. It should be possible to add a write ahead log too, by batching multiple insertions in a transactional (and async) way if needed (but I suspect not?).

Right now, writes require a lock, I'm not sure if there is any advantage in more coarse-grained locking, probably logging/batching is enough for writes. Readers are not blocked by writes in progress, there is only one lock in reads, that it's only taken for a brief time by a finished writer to add a new `current_version`. It may be possible to get rid of that lock (I don't know how though).

## Layout

The layout can be changed easily, but the pages are interpreted the following way

| page_tag (internal, leaf) | n || key1, key2, ... key_n || childs/values |

where childs is basically an array of n+1 elements and values is an array of n elements.
values are u64 values, representing offsets in the flatfile. And childs are pointers to pages of the tree

*n* can be computed from the *page_size*, in order to fit all the keys possible per block (keys are fixed size)