# Rusk-VM Persistence Interface

## NetworkState Instance Methods

- persist<P: AsRef<Path>>(path: P, compact: bool): Result<(), PersistError>

## NetworkState Static Constructor Methods

- restore<P: AsRef<Path>>(path: P): Result<NetworkState, PersistError>

## Notes

### Compacting
When parameter `compact` is true, compacting will be performed.
This means the following procedure will be executed:
- global unarchive operation will be performed
- network state will be stored to `path`

### Path
<P: AsRef<Path>> or impl AsRef<Path> is a more universal type than PathBuf used so far

### NetworkStateId
Use of NetworkStateId is hidden from the user.
It contains two OffsetLen elements for two contract maps, and it is stored
as file in a serialized form at the same location as the `path`.
Hence, there is no need to deal with it explicitly.
