`haitaka-types` `ハイタカ型`

## Internal package for `haitaka`

This package defines the core data types used in `haitaka`. It's not intended
as stand-alone library (though nothing will prevent you from using it like that). 

Splitting off the data types into a separate crate allows `haitaka` to run a
build script to generate slider move hash tables used in move generation based on
[magic bitboard](https://analog-hors.github.io/site/magic-bitboards/). Without a 
separate crate to run against the build script would have to duplicate quite a bit
of code or the library would need a separate initialization function -- either as
 a hidden, lazy initialization or as an explicit initialization step. Both those 
 alternatives are less than ideal.

Setting up a separate crate also ensures a clearer separation of concerns making
the codebase easier to maintain, test, and extend.