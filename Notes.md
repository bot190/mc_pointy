# Verifying information with the [NBT](https://github.com/twoolie/NBT) python library

## Region Header:
The region header format is 8192 bytes long. The first 4096 bytes correspond to chunk locations in the region file. The (X, Z) coordinates can be used to find the entry in this table with: `4 * ((x & 31) + (z & 31) * 32)`

Locations consist of 4 bytes per chunk
- The 3 most significant bits (on the left) make up the offset in 4kb sectors
- The LSB (right most byte) is the # of sectors of chunk data

The next 4096 bytes are used to store 4 byte timestamps of the last update time for each chunk.

    for index in range(0, 4096, 4):
         x = int(index//4) % 32
         z = int(index//4)//32
         m = region.metadata[x, z]
         if m.blockstart != 0:
                 print("Start: {} Length: {}".format(m.blockstart, m.blocklength))

## Chunk Data
Each chunk has a 5 byte header before the compressed NBT data.
- The first 4 bytes represent the length of chunk data
- The 5th byte represents the compression type

1. GZIP (Unused)
2. ZLIB