[![Coverage Status](https://coveralls.io/repos/github/gnieto/abxml-rs/badge.svg?branch=develop)](https://coveralls.io/github/gnieto/abxml-rs?branch=develop)
[![Build Status](https://travis-ci.org/gnieto/abxml-rs.svg?branch=develop)](https://travis-ci.org/gnieto/abxml-rs)

# ABXML

ABXML is a library that is able to decode APK files in Rust.

The code is deeply inspired on Apktool: Without it, this library wouldn't exist.

## Quick example

TODO

## Testing

To prepare the test environment the following tools should be installed on your path:

1. Rust
2. Apktool
3. xmllint
4. colordiff

After that, you should be able to run:

```
./script/compare_apk.sh PATH_TO_APK
```

You should see, per each file, if the output of the library matches with the output of Apktool.
In case that there is some difference, it will print the diff of the outputs.