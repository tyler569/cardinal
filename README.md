# cardinal

Cardinal is an experimental operating system concept built off Rust's
support for asynchronus programming.

The Cardinal system currently does not implement multithreading at all;
rather the kernel and userland each run an async executor which
schedules tasks.

## Project map

- `interface` contains definitions the kernel needs to export for
  userland.
- `allocator` contains a memory allocator that is used by both the
  kernel and userland.
- `kernel` contains the privileged system itself.
- `userland` contains the non-privileged system library and user
  programs.
