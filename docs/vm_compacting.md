# Persisting the VM State

## Introduction

Persisting state of VM consists of 2 parts:
1) persisting the database of contracts - a map from contract ids to corresponding contracts
2) persisting states of contracts

Second part is much harder to implement, as types of contracts' states are not easily accessible
from the host code. What the host code sees as contract' state is an array of bytes.
Hence, to store contract state one needs to ask the contract for help in the process.

## Short Explanation of the Compacted Persistence
Currently, contract's state is stored in two places - in contract's array of bytes called 'state',
and in the host store. Since all contracts belong to a network state, when network state is persisted,
VM stores the 'state' byte arrays of contracts first. The part in the host store needs to be persisted in a separate step,
although both are usually stored together, one after another.

Contract state is added to the host store upon contract's method return - when state is serialised.

Since state is added to the store upon return from methods, if contract methods are called many times,
state is added in multiple small incremental steps, each creating some memory references.
In time, contract state becomes dispersed along a large chunk of memory. Some old references are not used anymore,
but there is no easy way to remove them.
For example, when calling a transaction which adds a single element on NStack, then after 100k of such calls we get more than
100 MB of memory usage, although the actual size of meaningful data is much smaller.

The solution to this issue is to store the state as-is, with no regard for old references, in a compacted way.
For that we need to load the entire network state into memory at some point, so that there are no storage references used at all,
and then store back such resolved form of the state.
State stored in such way, after, say, 100k calls to a transaction performing a single push, consumes only around 4.7 MB of memory
rather than 100 MB.
Caveat is that in order to store the state as-is, we need to force all elements to be in an unarchived state.
To be able to do that, we need to know elements' types. The type is not visible from the host, hence, a hookup is needed so
that in-memory enforcing part is done by the contract, as only contracts know their states' types.
This requires adding a dedicated method to each contract, named "unarchive", which have the "state-un-archiving" code.
That code would enforce the in-memory location of state elements.

## Compacting Procedure Step by Step

Prerequisites:
1) VM persisted in a usual way, in a folder SRC_STORAGE
2) A file containing persistence id located at filesystem path SRC_PERSISTENCE_ID_FILE
3) Some empty folder TRG_STORAGE

### Step 1
Restore network state object S based on SRC_STORAGE and SRC_PERSISTENCE_ID_FILE

### Step 2
For every contract in S perform transaction "unarchive"

### Step 3
Create an empty target store T from TRG_STORAGE

### Step 4
Persist S to T and store new persistence id into a new file TRG_PERSISTENCE_ID_FILE.

### Step 5
Use TRG_STORAGE and TRG_PERSISTENCE_ID_FILE as the new, much smaller (compacted) storage.
You can discard SRC_STORAGE and SRC_PERSISTENCE_ID_FILE.

In order to perform the above procedure, you need to have a version of VM
in which `get`s read from the source store and `put`s write to the target store.
Hence, network state should support two stores, a source store and a target store.
During normal operation of the VM, source and target stores should be the same.
Only during the periodical compacting operation, target store will differ from
the source store.

## Discussion

It is possible to implement compacting procedure using just one store, yet it would
be tricky. There is a need for a cutoff in case there is only one store. Also, if the compacting
procedure fails for some reason, it could permanently damage the store. The above approach allows
for validating the new compacted store before removal of the old store.

We could envision having two storage locations, and alternating between them every hour or so.
This way we'd delete old storage only after a period of time of using the new storage, which would
give us confidence that the new storage is not corrupted.

I am not sure if this compacting approach can be confined within Microkelvin, the approach
relies on being able to read the old "big" storage and write into the new "small" storage, while
calling special contract method. Making this work in Microkelvin would require having
some knowledge about all objects, yet Microkelvin does not have access to
our network state. Passing network state to Microkelvin would take some duplication of code
and data, and some complication. This is an open point for a discussion.

## Measurements

For example, for contract `stack`

Node size is 136 (one node has slots for 4 elements)

not compacted:

| stack size     | nodes   |head offset     |head length     |origin offset   |origin length   |storage size
| :------------: | :-----: | :------------: | :------------: | :------------: | :------------: | :--------: |
|4               | 1       | 0              | 74320          | 74320          | 74320          | 148640
|5               | 2       | 272            | 74320          | 74592          | 74320          | 148912
|6               | 3       | 408            | 74320          | 74728          | 74320          | 149048
|7               | 4       | 544            | 74320          | 74864          | 74320          | 149184
|8               | 5       | 680            | 74320          | 75000          | 74320          | 149320
|9               | 7       | 952            | 74320          | 75272          | 74320          | 149592
|128             | 339     | 46104          | 74320          | 120,424        | 74320          | 194744
|512             | 1874    | 254,864        | 74320          | 329,184        | 74320          | 403504
|65536           | 458745  | 62,389,320     | 74320          | 62,463,640     | 74320          | 62,537,960

compacted:

| stack size     | nodes   |head offset     |head length     |origin offset   |origin length   |storage size
| :------------: | :-----: | :------------: | :------------: | :------------: | :------------: | :--------: |
| 4              | 1       | 0              | 74320          | 74320          | 74320          | 148640
| 5              | 2       | 272            | 74320          | 74592          | 74320          | 148912
| 6              | 2       | 272            | 74320          | 74592          | 74320          | 148912
| 7              | 2       | 272            | 74320          | 74592          | 74320          | 148912
| 8              | 2       | 272            | 74320          | 74592          | 74320          | 148912
| 9              | 3       | 408            | 74320          | 74728          | 74320          | 149048
| 128            | 42      | 5712           | 74320          | 80032          | 74320          | 154352
| 512            | 170     | 23120          | 74320          | 97440          | 74320          | 171760
| 65536          | 21844   | 2,970,784      | 74320          | 3,045,104      | 74320          | 3,119,424

### How to interpret the measurements

The above data comes from testing a contract whose state is a stack.
Contract method `push` is being called as many times as the `stack size` in the first column of the table.
The method pushes exactly one number onto the stack.
After transacting the `push` method N times, test persists the storage on disk. First the head contracts map is persisted,
then the origin contracts map is persisted.
As we can see, size of both maps is always the same, 74320. Offset of the second map is always equal to the offset of the first
map plus its length. So the most interesting information is in the offset of the head map - column marked as `head offset`.
From the head offset we can learn the size of storage used when serializing the stack. We notice that the offset is always a multiple
of 136, which happens to be the size of one nstack node when the element is of type `u64`.
We can see that up to 4 stack elements, storage size is zero for both not compacted and compacted data.
With the 5th element, storage size is 272 which is 136*2, as two nodes are stored.
Then from the 6th element upwards, in a non-compacted data we can see that one node is added at each transaction, thus
storage size grows in a non-proportional way to the number of elements. That is caused by the fact that previous nodes
are serialized and new element is always added to the new node rather than a slot in existing node.
We can see that compacted version does not have this behaviour, and nodes are fully utilized by it.
With the stack size of 64k elements, storage size difference is quite significant, 62MB versus 3MB.

## More Discussion Points

Scenario that we tested is an extreme case, in which a small change is performed a massive number of times.
We could think of some way of improving memory efficiency for this particular scenario, yet in practice
other scenarios might be more prevalent - like, for example, several contracts alternating access to the store
and thus entangling the links and causing excessive memory usage.
It seems that given a number of scenarios for which we provide alleviating schemes, it will always be possible
to come up with a scenario that will add one extra link that will undo the fix. So it looks like there is
no replacement for some global scheme, like un-archiving everything and archiving again.
Another approach would be to make store not global, but rather contract specific. This would eliminate
the possibility of inter-contract entanglements, and leave us with a much simpler problem of single-contract
storage. This would also have a consequence of opening the possibility of storing entire state on blockchain,
rather than only the tip of it.

### Regarding the initiative of moving storage compacting code to microkelvin:
Current compacting scheme relies on un-archiving all data, for which we need to have access to all idents
of the data stored, or at least to the top idents for known structures. These top idents are contained
in the network state and hence are not accessible to microkelvin. So the driver of the process needs to be in rusk-vm.
I'd argue that only the un-archiving code can actually be moved to microkelvin, so we could ask, for example,
a dusk-hamt map to unarchive itself. Other code has a driving nature, and it'd need to stay in rusk-vm.
Should we come up with a different compacting scheme, the above reasoning may be invalidated.
