## Example repo

This repository shows how the tool can be used to watch a plaintext notes repository. 

### Usage

Initialize the example repo:
```sh
cd example
git init && \
  git add -A && \
  git commit -am "initial commit"
```

Run gitwatch, using the configuration from [gitwatch.yml](gitwatch.yml)
```sh
cargo run watch --log-level=debug
```
