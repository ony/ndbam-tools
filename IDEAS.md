## Shared ownership

Use filesystem backed index based on sha1 hash from canonical path.

```
data
  +- abc
  +- def

shared_paths
  +- 8040f3f3 (/usr/shared/info-index)
	 +- abc -> ../../data/abc
	 +- def -> ../../data/def
  +- 84e617a4 (/usr/shared/man-index)
	 +- abc -> ../../data/abc
	 +- def -> ../../data/def
```
