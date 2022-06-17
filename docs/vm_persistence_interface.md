# Rusk-VM Persistence Interface

## NetworkState Static Constructor Methods
- `pub fn create<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
- `pub fn restore<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
- `pub fn compact<P>(from_path: P, to_path: P, gas_meter: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path>`

- goal:
- `pub fn compact<P>(to_path: P, gas_meter: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path>`

## NetworkState Instance Methods
- `pub fn persist<P>(&mut self, path: P) -> Result<(), VMError> where P: AsRef<Path>`

- goal:
- `pub fn persist<P>(&mut self) -> Result<(), VMError>`

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
Network state can be persisted to a give location, note that only network state id will be persisted
to the `path` location, while network state' store will be persisted its initial location, the one with
which the state was created or restored from.
TODO - this is confusing and needs to be fixed. We need to be able to persist in place, hence,
we need to store network state id at the same location as store' location.

### Path
<P: AsRef<Path>> is a more universal type than PathBuf used so far

### NetworkStateId
Use of NetworkStateId is hidden from the user.
It contains two OffsetLen elements for two contract maps, and it is stored
as file in a serialized form at the same location as the `path`.
Hence, there is no need to deal with it explicitly.
