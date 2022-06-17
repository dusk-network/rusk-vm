# Rusk-VM Persistence Interface

## NetworkState Static Constructor Methods
- `pub fn create<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
(will create NetworkState with store whose "native" path is `path`)
- `pub fn restore<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
- `pub fn compact<P>(from_path: P, to_path: P, gas_meter: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path>`

## NetworkState Instance Methods
- `pub fn persist<P>(&mut self, path: P) -> Result<(), VMError> where P: AsRef<Path>`
  (will persist store to its "native" path and the resulting NewtworkStateId to `path` )


## Notes

### Creating
Network state can be created from scratch, given an existing file system location `path`.
Store needed for subsequent contract operations can then be obtained via `get_store_ref()`.

### Restoring
Network state can be restored from a disk location, especially, this can be done
after system restart or after compacting.

### Compacting
Compact method executes the following procedure:
- extracts network state id from `from_path`
- restores network state using network state id and `from_path`
- performs global unarchive operation on contracts' states (by invoking transaction `unarchive` on all contacts)
- stores the network state to a new file system location at `to_path`

After compacting, user can invoke `restore` on `to_path` to obtain new network state.
The location at `from_path` can then be discarded.
Both locations `from_path` and `to_path` need to exist before the call and have the necessary write permissions.

### Persisting
Network state can be persisted to a given location, note that only network state id will be persisted
to the `path` location, while network state' store will be persisted its initial location, the one with
which the state was created or restored from.
When `persist` is used after `create` or `restore`, the same `path` should be used for `persist` as for
`create`/`restore`.

### Path
<P: AsRef<Path>> is a more universal type than PathBuf used so far

### NetworkStateId
Use of NetworkStateId is hidden from the interface's user.
It contains two OffsetLen elements for two contract maps, and it is stored
as a file in serialized form at the same location as the `path`.
Hence, there is no need to deal with it explicitly.
