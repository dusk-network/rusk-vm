# Rusk-VM Persistence Interface

## NetworkState Instance Methods

- `pub fn persist<P>(&mut self, path: P) -> Result<(), VMError> where P: AsRef<Path>`

## NetworkState Static Constructor Methods
- `pub fn create<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
- `pub fn restore<P>(path: P) -> Result<NetworkState, VMError> where P: AsRef<Path>`
- `pub fn compact<P>(from_path: P, to_path: P, gas_meter: &mut GasMeter) -> Result<(), VMError> where P: AsRef<Path>`

## Notes

### Compacting
Compact method executes the following procedure:
- extracts network state id from `path`
- restores network state using network state id and `path`
- performs global unarchive operation on contracts' states (by invoking transaction `unarchive` on all contacts)
- stores the network state to a new file system location (new path)
- returns the new path

### Path
<P: AsRef<Path>> or impl AsRef<Path> is a more universal type than PathBuf used so far

### NetworkStateId
Use of NetworkStateId is hidden from the user.
It contains two OffsetLen elements for two contract maps, and it is stored
as file in a serialized form at the same location as the `path`.
Hence, there is no need to deal with it explicitly.
