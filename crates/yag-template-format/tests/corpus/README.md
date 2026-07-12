# Formatter corpus

The `yagpdb-cc` submodule pins the community-maintained
[YAGPDB custom-command collection](https://github.com/yagpdb-cc/yagpdb-cc).
The corpus test formats every `src/**/*.tmpl` file from that repository.

Initialize the corpus after cloning with:

```sh
git submodule update --init --recursive
```

To refresh the corpus, update the `yagpdb-cc` submodule deliberately and
commit the resulting gitlink change.
