# Contributing

There is a feature missing? A bug you have noticed? Some inconsistencies? **Contributions are welcome, and are encouraged!**

## Guidelines

We welcome your contributions, and we love to keep our code standards high. So, there are a few key guidelines that you should follow for smooth sailing:

- All our code is formatted using rustfmt. Please, run `cargo fmt` before committing your changes.
- Make sure all of the tests pass with `cargo test`, as this would prevent us from merging your changes.
- Make sure that clippy does not complain with `cargo clippy --all-targets --all-features --workspace -- -D warnings -D clippy::all`

## Review

Once you submit a pull request, one of the maintainers will have a look at it, and give you some feedback. If everything looks alright, we will be almost ready to merge it in! If not, the maintainer will point you to the right direction where things may need changing in the code.

## Merging

Once the code is ready, the last step is merging. There are only 2 important things that you need to confirm:

- That the code is yours
- And that you agree with the project's license terms to be applied to the entire pull request.

By default, we will go by the assumption that those 2 points are true, but it would still be nice that you confirmed those. And sometimes, we may ask you to do so, just to be sure.

Ultimately, unless you state otherwise, the merged code will be licensed under the current license of the project.

Anyways, thanks for giving this a read, and happy hacking!
