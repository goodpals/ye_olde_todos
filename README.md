# 𝓨𝓮 𝓞𝓵𝓭𝓮 𝓣𝓸𝓭𝓸𝓼
A simple tool for finding `TODO` comments in code, and sorting them by age.  

## Usage
Install with `brew install goodpals/goodpals/yot` (may need to `brew tap goodpals/goodpals` first), or by cloning this repo and running `cargo install --path .`.

Run `yot` in a directory that's part of a git repository. Can also take a path with `yot -p /path/to/thing`.  
It won't work outside of git repos.

It currently finds comments like `// TODO` or `# TODO`. It works by using `git blame`.
