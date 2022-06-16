# Rusk-VM Persistence Interface

## NetworkState Instance Methods

- pub fn persist<P: AsRef<Path>>(&mut self, path: P) -> Result<(), VMError>

## NetworkState Static Constructor Methods

- pub fn compact<P: AsRef<Path>>(path: P, gas_meter: &mut GasMeter) -> Result<PathBuf, VMError>

- pub fn restore<P: AsRef<Path>>(path: P) -> Result<NetworkState, VMError>

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
