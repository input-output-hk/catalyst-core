# Implementation plan

## Phase 1 - Initial container

The first iteration is a container image which incorporates all the tools, and answers heartbeat.
It also exposes basic metrics and logs.

## Phase 2 - (Final Phase - 1)

1. Each major step in the Leader 0 and other Leaders flow can be broken into discreet functions.
   1. There is some redundancy between these functions, so that should be exploited.
2. They can be 1 ticket worth of output each.

## Final Phase

1. Integrating all steps.
2. Testing with various boundary conditions
3. Iterating on process flow until deployment is stable and reliable.
