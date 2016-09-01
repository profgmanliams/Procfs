//
// Print the physical memory page with the most references
//
// Require CAP_SYS_ADMIN privilege, or root
//
// Sample output:
//
// Found RAM here: 0x1000-0x9fbff
// Lots of references to this locations: addr=0x9d000, pfn=157, refs=0
// Found RAM here: 0x100000-0xdffeffff
// Lots of references to this locations: addr=0x81ba3000, pfn=531363, refs=128
// Found RAM here: 0x100000000-0x11fffffff
// Lots of references to this locations: addr=0x1b575000, pfn=111989, refs=134
//

use procfs::prelude::*;

fn main() {
    if !rustix::process::geteuid().is_root() {
