# fish/

Place fish species folders here. Each species folder should contain optional `left` and `right` subfolders with CSV frames (same format used previously).

Example layout:

    src/fish/
      tuna/
        left/
          frame1.csv
          frame2.csv
        right/
          frame1.csv
          frame2.csv
      goldfish/
        right/
          frame1.csv

The loader will concatenate frames from each species' `right` folders into the global right-facing frames list, and similarly for `left`.
