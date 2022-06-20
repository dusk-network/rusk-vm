# Rusk-VM Persistence Interface

## Backend Constructor Methods
- `pub fn new<P>(path: P) -> Result<Backend, VMError> where P: AsRef<Path>`
- `pub fn restore<P>(path: P) -> Result<Backend, VMError> where P: AsRef<Path>`
- `pub fn compact<P>(from_path: P, to_path: P, gas_meter: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path>`

## Backend Instance Methods
- `pub fn persist(&mut self) -> Result<(), VMError> where P: AsRef<Path>`


## Notes

### Creating
Backend containing network state instance can be created from scratch via method `new`.
Restore is also a way to create backend.
Location `path` needs to exist before the call and have necessary permissions.

### Restoring
Backend can be restored from a disk location, this can be done, e.g.,
after system restart or after compacting.

### Compacting
Compacting method executes the following procedure:
- extracts network state id from `from_path`
- restores network state using network state id and `from_path`
- performs global unarchive operation on contracts' states (by invoking transaction `unarchive` on all contacts)
- stores the network state to a new file system location at `to_path`
- stores new network state id at the same location

After compacting, user can invoke `restore` on `to_path` to obtain a new compacted instance of the Backend.
The location at `from_path` can then be discarded.
Both locations `from_path` and `to_path` need to exist before the call and need to have necessary permissions.

### Persisting
Backend instance can be persisted to its location.

### NetworkStateId
The use of NetworkStateId is hidden, so that user of the interface does not need to be aware of it.
NetworkStateId contains OffsetLen elements for contract maps, and it is stored
as a file in serialized form at the same location as the corresponding store.
There is no need to deal with it explicitly.
