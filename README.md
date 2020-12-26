# cdl
A command-line application for downloading Minecraft mods with dependencies.

## Using
`cdl --help` will print relevant information about using the program. The following shows some examples of how you may want to use it.

### Examples
`cdl jei` will search for mods containing the phrase "jei" (case-insensitive), using the default values. The values can be changed in `~/.config/cdl/default.toml`, and the default is to search for Forge mods for 1.16.4 sorting by popularity and showing 9 results per query.

`cdl jei -v 1.12.2` specifies to search for game version 1.12.2. This overrides the toml-file, like all options do.

`cdl "applied energistics 2" -l fabric` narrows the search to Fabric mods only. Can be either 'forge', 'fabric', or 'both'.

`cdl jei -a 40` specifies to include up to 40 results on search.

`cdl -s created jei` specifies to sort the results by the date the mod was created.

`cdl -a 1 -v 1.7.10 -l forge -s updated ars` asks for the most recently updated mod including "ars" for Forge and Minecraft 1.7.10.

`cdl -g Foo/Bar` clones the repository at `https://github.com/Foo/Bar.git` and asks you to choose a branch before attempting to execute `./gradlew` in order to compile the mod from source. Once finshed, it asks which file(s) you want to copy.
