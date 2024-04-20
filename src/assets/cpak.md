# Cpak files
## Cobalt pak files
### Cobalt pak files are used to store the game's assets, such as textures, models, sounds, and more.

## How it works
The engine will read every pak file at startup and store the file table in memory. When the engine needs to load a file, it will search for the file in the file table and load the file data from the pak file. If the file is not found in the file table, the engine will search for a file.

## Structure
### Header
- `char[4]` - Magic number, always "CPAK"
- `uint32` - Version
- `uint32` - Number of files
- `uint32` - Offset to file table
- `uint32` - Offset to file data

### File table
#### Serialized hashmap of file paths and offsets
#### Starts with the size of the hashmap data
- `uint32` - Size of the hashmap data
- `utf yaml string` - Hashmap data