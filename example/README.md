## Example repo

This repository shows how gitwatch can be used to watch a local plaintext notes repository and automatically generate commit messages using [aichat](https://github.com/sigoden/aichat) via a custom [commit message script](./gitwatch-commit-message.sh). 

<img src="../docs/example.png" alt="Example use case">

### Usage

Initialize the example repo:
```sh
cd example
git init && \
  git add -A && \
  git commit -am "initial commit"
```

Run gitwatch (the [local config file](./gitwatch.yaml) is automatically used):
```sh
gitwatch watch --log-level=debug
# or
gitwatch watch --log-level=debug
```
