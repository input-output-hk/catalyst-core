# E2E tests

There should to be testing against a known data set.
Currently this does not exist.

We should get a dump of pre-prod up to a known date.
Then use earthly to populate a test postgres instance with that dump (it can be abridged to just relevant data).
The tests can then be run against that.
