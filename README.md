## An Easy NodeJS (version) Manager (ENM)

### Use `enm` to manage multiple NodeJS versions on your system easily

## Get Binary
### Build binary locally using Rust

1.  `git clone https://github.com/codelinter/enm && cd enm`
2.  `cargo install --path .`

### Get the binary directly from the releases section

## $PATH
### Put the binary in the $PATH, so that its available system wide

## Quick Start

1. Source environment

   `eval "$(enm source)"`
   
      Or if your NodeJS project root workspace contains
      a `.nvmrc` or `.node-version` file, with version number inside of it, then use the below command to trigger an auto switch
   
   `eval "$(enm source --on-enter)"`

2. Now use any node version. The below command will now download the latest NodeJS 18 version if not already on your system

   `enm switch v18` or `enm switch 18` or `enm switch 18.1`

3. Install NodeJS version

   `enm install 20` or `enm install 20.1`

## Recommended way

### Quick start works fine, but you will have to run the `eval` command every time you open a new shell

### Instead, add that command in your `$PROFILE` or `$SHELL`

### On Linux/Mac 
1. Use `~/.bashrc`,  `~/.zshrc` or `~/.profile`

```bash
   eval "$(enm source --on-enter)"
```

### On Windows 
### Now using any editor, open `$profile`

### Ex: using Powershell
   ```ps1
    notepad $profile
   ```

### Add this at the end of that file

   ```ps1
   enm source --on-enter --shell powershell | Out-String | Invoke-Expression
   ```