hyttpd
======

I have a tiny little Ruby script that starts WEBrick in the current
directory on a high port.  It's useful for viewing offline websites,
say, when you're doing web development... or viewing the Rust docs
offline. 

I felt kinda dirty using a Ruby script for viewing Rust docs, so I
wrote this.  It's tiny, and crap, but it works.

### Bugs^WFeatures

 - all access and error logging is to stdout.
 - directory indexes are in whatever order read_dir wants
 - unstable features; hope you've got nightly!

### License

Copyright (c) 2015 Jashank Jeremy.

Licensed under the Apache License, Version 2.0 or the MIT license, at
your option. See LICENSE-APACHE and LICENSE-MIT for details.
